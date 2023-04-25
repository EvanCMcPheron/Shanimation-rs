use super::prelude::*;
use std::fs::DirBuilder;
use std::io::Write;

use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use super::encoding::rgba_to_yuv;

use crate::encoding::RateControlMode;
use image::RgbaImage;
use openh264::encoder::{Encoder, EncoderConfig};

use fast_inv_sqrt::InvSqrt64;

use crossterm::{
    cursor::{Hide, MoveToPreviousLine},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{
        BeginSynchronizedUpdate, EndSynchronizedUpdate,
    },
};
use std::cmp::{max, min};
use std::time::Instant;

const LENGTH_ADJUSTMENT_FACTOR: f64 = 5.0 / 6.0; //250.0 / 295.0;

#[derive(Clone)]
pub struct Img {
    pub dimensions: Point<usize>,
    pub image: RgbaImage,
}

impl Img {
    pub fn new(dimensions: Point<usize>) -> Self {
        Self {
            dimensions,
            image: RgbaImage::new(dimensions.x as u32, dimensions.y as u32),
        }
    }
    pub fn get_pixel(&self, point: Point<usize>) -> Rgba<u8> {
        *self.image.get_pixel(point.x as u32, point.y as u32)
    }
    pub fn set_pixel(&mut self, point: Point<usize>, color: Rgba<u8>) {
        self.image.put_pixel(point.x as u32, point.y as u32, color);
    }
}

#[derive(ErrorStack, Debug)]
#[error_message("An error occured while rendering a scene")]
pub enum SceneRenderingError {
    FileWritingError,
    FrameRenderingError,
    EncodingError,
    FFMPEGError,
    Crossterm,
}

#[derive(Clone)]
pub struct Scene {
    children: Vec<Arc<RwLock<Renderable>>>,
    resolution: Point<usize>,
    fps: usize,
    length: Duration,
    rate_control_mode: RateControlMode,
}

impl Scene {
    pub fn clone_entire(&self) -> Self {
        let mut new_self = self.clone();
        let children = self
            .children
            .iter()
            .map(|c| c.read().unwrap().clone_entire())
            .map(|c| Arc::new(RwLock::new(c)))
            .collect::<Vec<_>>();
        new_self.children = children;
        new_self
    }
    pub fn builder() -> SceneBuilder {
        SceneBuilder {
            children: Some(vec![]),
            resolution: Some(Point::new(1280, 720)),
            fps: Some(30),
            length: None,
            rate_control_mode: RateControlMode::Bufferbased,
        }
    }
    pub fn get_children(&self) -> &Vec<Arc<RwLock<Renderable>>> {
        &self.children
    }
    pub fn get_children_mut(&mut self) -> &mut Vec<Arc<RwLock<Renderable>>> {
        &mut self.children
    }
    pub fn add_child_simple(&mut self, child: Renderable) -> Arc<RwLock<Renderable>> {
        let s = Arc::new(RwLock::new(child));
        self.children.push(s.clone());
        s
    }
    pub fn add_child(&mut self, child: Arc<RwLock<Renderable>>) {
        self.children.push(child);
    }
    fn render_frames(&self) -> Result<Vec<u8>, SceneRenderingError> {
        println!("Rendering video...\n");
        //figure out frame count, with matching duration to send to behaviour and shader
        let max_frames = self.length.as_secs() as usize * self.fps;
        let seconds_per_frame = 1.0 / self.fps as f64;
        let mut video_bytes: Vec<u8> = vec![];
        let mut encoder = Encoder::with_config(
            EncoderConfig::new(self.resolution.x as u32, self.resolution.y as u32)
                .max_frame_rate(self.fps as f32)
                .rate_control_mode(self.rate_control_mode),
        )
        .into_report()
        .change_context(SceneRenderingError::EncodingError)
        .attach_printable_lazy(|| "Failed to create encoder")?;
        let mut start = Instant::now();
        let tp = threadpool::ThreadPool::new(8);
        let mut receivers = vec![];
        let mut encoded_count = 0;
        let rendered_count = Arc::new(RwLock::new(0));
        //for each frame, run render frame
        for (frame_indx, time) in (0..max_frames)
            .map(|i| Duration::from_secs_f64(i as f64 * seconds_per_frame))
            .enumerate()
        {
            execute!(
                std::io::stdout(),
                BeginSynchronizedUpdate,
                Hide,
                MoveToPreviousLine(1),
                SetForegroundColor(Color::Blue),
                Print(format!("Runnning Behaviours")),
                ResetColor,
                Print(format!(
                    " | Processed {} / {} frames",
                    frame_indx, max_frames
                )),
                SetForegroundColor(Color::Blue),
                Print(format!(
                    " ({:.0}%)",
                    (frame_indx as f64 / max_frames as f64) * 100.0
                )),
                ResetColor,
                Print(" | ".to_owned()),
                ResetColor,
                Print("\n".to_owned()),
                //ScrollDown(3),
                EndSynchronizedUpdate,
            )
            .into_report()
            .change_context(SceneRenderingError::Crossterm)
            .attach_printable_lazy(|| "Failed to execute crossterm")
            .unwrap();
            self.run_behaviours(time);
            let cloned_scene = self.clone_entire();
            let cloned_count = rendered_count.clone();
            let (sender, rec) = std::sync::mpsc::channel();
            receivers.push(rec);
            tp.execute(move || {
                sender
                    .send((
                        frame_indx,
                        rgba_to_yuv(
                            cloned_scene
                                .render_frame(frame_indx, time)
                                .change_context(SceneRenderingError::FrameRenderingError)
                                .attach_printable_lazy(|| {
                                    format!(
                                        "Failed to render frame {} at time {} seconds",
                                        frame_indx,
                                        time.as_secs_f64()
                                    )
                                })
                                .unwrap(),
                        ),
                    ))
                    .expect("Failed to send frame through mpsc channel");
                *cloned_count.write().unwrap().deref_mut() += 1;
            });
        }
        let mut has_finished_frames = false;
        let mut encoded_at_start = 0;
        for (frame_indx, receiver) in receivers.iter().enumerate() {
            let (rendered_indx, frame) = receiver.recv().unwrap();
            assert_eq!(frame_indx, rendered_indx);
            encoder
                .encode(&frame)
                .into_report()
                .change_context(SceneRenderingError::EncodingError)
                .attach_printable_lazy(|| "Failed to encode frame")?
                .write_vec(&mut video_bytes);
            encoded_count += 1;
            if tp.active_count() == 0 {
                if has_finished_frames == false {
                    has_finished_frames = true;
                    start = Instant::now();
                    encoded_at_start = encoded_count;
                }
                let eta = Duration::from_secs_f64(
                    start.elapsed().as_secs_f64()
                        * (((max_frames - encoded_at_start) as f64
                            / std::cmp::max(1, encoded_count - encoded_at_start) as f64)
                            - 1.0)
                            .abs(),
                );
                execute!(
                    std::io::stdout(),
                    BeginSynchronizedUpdate,
                    Hide,
                    MoveToPreviousLine(1),
                    SetForegroundColor(Color::Blue),
                    Print(format!("Finished rendering frames, still encoding video")),
                    ResetColor,
                    Print(format!(
                        " | Encoded {} / {} frames",
                        encoded_count, max_frames
                    )),
                    SetForegroundColor(Color::Blue),
                    Print(format!(
                        " ({:.0}%)",
                        (encoded_count as f64 / max_frames as f64) * 100.0
                    )),
                    ResetColor,
                    Print(" | ".to_owned()),
                    SetForegroundColor(Color::Green),
                    Print(format!(
                        "ETA: {}:{}",
                        eta.as_secs() / 60,
                        eta.as_secs() % 60
                    )),
                    ResetColor,
                    Print("\n".to_owned()),
                    //ScrollDown(3),
                    EndSynchronizedUpdate,
                )
                .into_report()
                .change_context(SceneRenderingError::Crossterm)
                .attach_printable_lazy(|| "Failed to execute crossterm")
                .unwrap();
            } else if let Ok(rc) = rendered_count.read() {
                let r_count = *rc.deref();
                let eta = Duration::from_secs_f64(
                    start.elapsed().as_secs_f64()
                        * ((max_frames as f64 / std::cmp::max(1, encoded_count) as f64) - 1.0),
                ); //minus one bcs x*a - x = x(a - 1)
                execute!(
                    std::io::stdout(),
                    BeginSynchronizedUpdate,
                    Hide,
                    MoveToPreviousLine(1),
                    Print(format!("Rendered frame {}/{}", r_count, max_frames)),
                    SetForegroundColor(Color::Blue),
                    Print(format!(
                        " ({:.0}%)",
                        (r_count as f64 / max_frames as f64) * 100.0
                    )),
                    ResetColor,
                    Print(format!(
                        " | Encoded {} / {} frames",
                        encoded_count, max_frames
                    )),
                    SetForegroundColor(Color::Blue),
                    Print(format!(
                        " ({:.0}%)",
                        (encoded_count as f64 / max_frames as f64) * 100.0
                    )),
                    ResetColor,
                    Print(" | ".to_owned()),
                    SetForegroundColor(Color::Green),
                    Print(format!(
                        "ETA: {}:{}",
                        eta.as_secs() / 60,
                        eta.as_secs() % 60
                    )),
                    ResetColor,
                    Print("\n".to_owned()),
                    EndSynchronizedUpdate,
                )
                .into_report()
                .change_context(SceneRenderingError::Crossterm)
                .attach_printable_lazy(|| "Failed to execute crossterm")
                .unwrap();
            }
        }
        Ok(video_bytes)
    }
    fn run_behaviours(&self, time: Duration) {
        let mut stack: Vec<(Point<isize>, Point<f64>, Arc<RwLock<Renderable>>)> = vec![]; 
        self.children
            .iter()
            .map(Clone::clone)
            .map(|c| (Point::new(0, 0), Point::new(1.0, 1.0), c))
            .for_each(|v| stack.push(v));
        //for each child, run their run their behaviour's process, then for every pixel, run their get_pixel (THIS CAN EASILY BE PARRELLELIZED) and overide the pixel on the main image buffer'
        while let Some((residual_offset, residual_scale, child)) = stack.pop() {
            let mut child = child.write().unwrap();
            //child.run_behaviour(time);

            // This scale should be multiplied against the dimensions when calculating the bottom right point (and also get passed to children), but only residual scale should be applied to the top left point.
            let next_scale = Point::new(
                residual_scale.x * child.params.scale.x,
                residual_scale.y * child.params.scale.y,
            );

            //This should be passed down to children AND used for coordinate calculations
            let next_offset = Point::new(
                residual_offset.x
                    + (child.params.position.x * self.resolution.x as f64 * residual_scale.x)
                        as isize,
                residual_offset.y
                    + (child.params.position.y * self.resolution.y as f64 * residual_scale.y)
                        as isize,
            );

            child
                .get_children()
                .iter()
                .map(Clone::clone)
                .map(|c| (next_offset, next_scale, c))
                .for_each(|c| stack.push(c))
                ;
            child.run_behaviour(time, self, next_offset);
        }
    }
    fn render_frame(
        &self,
        _frame_indx: usize,
        time: Duration,
    ) -> Result<RgbaImage, SceneRenderingError> {
        //create an empty rgba image buffer
        let mut img_buffer = Img::new(self.resolution);
        //'recursively' iterate through all children of the scene, and their children, (run from top down)
        let mut stack: Vec<(Point<isize>, Point<f64>, Arc<RwLock<Renderable>>)> = vec![];
        self.children
            .iter()
            .map(Clone::clone)
            .map(|c| (Point::new(0, 0), Point::new(1.0, 1.0), c))
            .for_each(|v| stack.push(v));
        //for each child, run their run their behaviour's process, then for every pixel, run their get_pixel (THIS CAN EASILY BE PARRELLELIZED) and overide the pixel on the main image buffer'
        while let Some((residual_offset, residual_scale, child)) = stack.pop() {
            let mut child = child.write().unwrap();
            //child.run_behaviour(time);

            // This scale should be multiplied against the dimensions when calculating the bottom right point (and also get passed to children), but only residual scale should be applied to the top left point.
            let next_scale = Point::new(
                residual_scale.x * child.params.scale.x,
                residual_scale.y * child.params.scale.y,
            );

            //This should be passed down to children AND used for coordinate calculations
            let next_offset = Point::new(
                residual_offset.x
                    + (child.params.position.x * self.resolution.x as f64 * residual_scale.x)
                        as isize,
                residual_offset.y
                    + (child.params.position.y * self.resolution.y as f64 * residual_scale.y)
                        as isize,
            );

            child
                .get_children()
                .iter()
                .map(Clone::clone)
                .map(|c| (next_offset, next_scale, c))
                .for_each(|c| stack.push(c));

            let abs_width = child.params.size.x * self.resolution.y as f64 * next_scale.x;
            let abs_height = child.params.size.y * self.resolution.y as f64 * next_scale.y;


            //For every pixel within the bounds of the shader, run the get_pixel fn and overide the pixel on the main image buffer
            let up_left_unchecked = Point::new(next_offset.x, next_offset.y);
            let down_right_unchecked = Point::new(
                up_left_unchecked.x
                    + (abs_width as isize),
                up_left_unchecked.y
                    + (abs_height as isize),
            );
            let up_left = Point::new(max(up_left_unchecked.x, 0), max(up_left_unchecked.y, 0));
            let down_right = Point::new(
                min(self.resolution.x as isize, down_right_unchecked.x),
                min(self.resolution.y as isize, down_right_unchecked.y),
            );

            let old_image = img_buffer.clone();

            ((up_left.y)..(down_right.y))
                .map(|y| {
                    ((up_left.x)..(down_right.x))
                        .map(|x| Point::new(x, y))
                        .collect::<Vec<Point<isize>>>() // *may* cause bottleneck
                })
                .flatten()
                .for_each(|p| {
                    let uv = to_uv(up_left_unchecked, down_right_unchecked, p)
                        .map_both(|v| v - 0.5)
                        .map_x(|x| x * (abs_width as f64 / abs_height as f64))
                        .to_polar()
                        .map_y(|y| y - child.params.rotation)
                        .to_cartesian()
                        .map_x(|x| x * (abs_height as f64 / abs_width as f64))
                        .map_both(|v| v + 0.5);
                    
                    let bounds_checked = uv.map_both(|v| (v >= 0.0 && v <= 1.0) as u8);

                    let c = Point::new(p.x as usize, p.y as usize);

                    if bounds_checked.x == 0 || bounds_checked.y == 0 {
                        img_buffer.set_pixel(c,Rgba([0,0,0,0]));
                        return ();
                    }
                    
                    let color = child.run_shader(&old_image, uv, time, next_offset);
                    let current_color = old_image.get_pixel(c);
                    let a = color.0[3];
                    let mixer = |i: usize| {
                        (color.0[i] as f32 / 255.0 * a as f32
                            + current_color.0[i] as f32 / 255.0 * (255.0 - a as f32))
                            as u8
                    };
                    let a_mixer = || {
                        a.checked_add(current_color.0[3].checked_mul(255 - a).unwrap_or(255))
                            .unwrap_or(255)
                    }; //Not sure if this is how alpha mixing SHOULD work, but it seems right?
                    let new_color = Rgba([mixer(0), mixer(1), mixer(2), a_mixer()]);

                    img_buffer.set_pixel(c,new_color);
                })
        }
        //write image buffer to file
        Ok(img_buffer.image)
    }
    /*
    fn post_process_frame() -> Result<(), SceneRenderingError> {
        In future, potentially add an optional post-processing feild to scene, which takes an takes the final image buffer and is free to operate on it any way it wants.
        That would be run in this fn, which would be run directly after the 'render_frame()' fn.
    }
     */
    fn compile_video(&self, bytes: Vec<u8>) -> Result<PathBuf, SceneRenderingError> {
        //Create Output dir if it doesn't exist
        if !Path::new("./output").exists() {
            DirBuilder::new()
                .recursive(true)
                .create("./output")
                .into_report()
                .change_context(SceneRenderingError::FileWritingError)
                .attach_printable_lazy(|| "Failed to create output directory")?;
        }
        //Generate an unused filename
        let mut output_filename = "./output/scene_0.mp4".to_owned();
        let mut i = 1;
        while Path::new(&output_filename).exists() {
            output_filename = format!("./output/scene_{}.mp4", i);
            i += 1;
        }
        let mut output_filename = std::path::PathBuf::from(output_filename);
        output_filename.set_extension("h264");
        //Use this command (add formatting)
        //self.run_ffmpeg_cmd(output_filename)
        let _file = std::fs::File::create(&output_filename);
        std::fs::write(&output_filename, bytes)
            .into_report()
            .change_context(SceneRenderingError::FileWritingError)
            .attach_printable_lazy(|| "Failed to write video file")?;
        Ok(PathBuf::from(output_filename))
    }
    fn run_ffmpeg_cmd(&self, path: PathBuf) -> Result<(), SceneRenderingError> {
        let glob_path = std::env::current_dir()
            .into_report()
            .change_context(SceneRenderingError::FFMPEGError)
            .attach_printable_lazy(|| "Failed to get current directory")?
            .as_os_str()
            .to_str()
            .ok_or(Report::new(SceneRenderingError::FFMPEGError))
            .attach_printable_lazy(|| "Failed to convert current directory to string")?
            .to_owned();

        let mut temp_mp4_path = path.clone().display().to_string();
        temp_mp4_path = temp_mp4_path.replace(".h264", "_temp.mp4");
        let mut mp4_path = path.clone();
        mp4_path.set_extension("mp4");

        let mut cmd = Command::new("ffmpeg")
            .current_dir(glob_path.clone())
            .arg("-i")
            .arg(format!("{}", path.display()))
            .arg(format!("{}", temp_mp4_path))
            .spawn()
            .into_report()
            .change_context(SceneRenderingError::FFMPEGError)
            .attach_printable_lazy(|| "Failed to convert video file")?;
        cmd.wait().unwrap();

        let mut cmd = Command::new("ffmpeg")
            .current_dir(glob_path)
            .arg("-i")
            .arg(temp_mp4_path.clone())
            .arg("-vf")
            .arg(format!(
                "setpts={}*PTS",
                (30.0 / self.fps as f64) * LENGTH_ADJUSTMENT_FACTOR
            ))
            .arg("-r")
            .arg(format!("{}", self.fps))
            .arg(mp4_path.display().to_string())
            .spawn()
            .into_report()
            .change_context(SceneRenderingError::FFMPEGError)
            .attach_printable_lazy(|| "Failed to adjust speed of video file")?;
        cmd.wait().unwrap();

        std::fs::remove_file(temp_mp4_path).unwrap();
        std::fs::remove_file(path).unwrap();

        //ffmpeg -reinit_filter 0 -f concat -safe 0 -i "frames/dict.txt" -vf "scale=1280:720:force_original_aspect_ratio=decrease:eval=frame,pad=1280:720:-1:-1:color=black:eval=frame,settb=AVTB,setpts=0.033333333*N/TB,format=yuv420p" -r 30 -movflags +faststart output.mp4

        //ffmpeg -f concat -safe 0 -i "frames/dict.txt" -vf "setpts=0.033333333*N/TB" -r 30 -movflags +faststart output.mp4
        /*let glob_path = std::env::current_dir()
            .into_report()
            .change_context(SceneRenderingError::FFMPEGError)
            .attach_printable_lazy(|| "Failed to get current directory")?
            .as_os_str()
            .to_str()
            .ok_or(Report::new(SceneRenderingError::FFMPEGError))
            .attach_printable_lazy(|| "Failed to convert current directory to string")?
            .to_owned();

        Command::new("ffmpeg")
            .current_dir(glob_path)
            .arg("-f")
            .arg("concat")
            .arg("-safe")
            .arg("0")
            .arg("-i")
            .arg("./frames/dict.txt")
            .arg("-filter")
            .arg(format!("setpts={h}*N/TB", h = 1.0 / self.fps as f32))
            .arg("-r")
            .arg(format!("{}", self.fps))
            .arg("-movflags")
            .arg("+faststart")
            .arg(path)
            .spawn()
            .into_report()
            .change_context(SceneRenderingError::FFMPEGError)
            .attach_printable_lazy(|| "Failed to concatinate frames into video through ffmpeg")?;
        */
        Ok(())
    }
    pub fn render(&self) -> Result<(), SceneRenderingError> {
        ffmpeg_sidecar::download::auto_download().unwrap();
        let bytes = self
            .render_frames()
            .attach_printable_lazy(|| "Failed to render frames")?;
        self.run_ffmpeg_cmd(self.compile_video(bytes)?)
    }
}

#[derive(ErrorStack, Debug)]
#[error_message("Not all builder requirements were fullfilled")]
pub struct SceneBuilderError;

pub struct SceneBuilder {
    children: Option<Vec<Arc<RwLock<Renderable>>>>,
    resolution: Option<Point<usize>>,
    fps: Option<usize>,
    length: Option<Duration>,
    rate_control_mode: RateControlMode,
}

impl SceneBuilder {
    pub fn with_resolution(&mut self, resolution: Point<usize>) -> &mut Self {
        self.resolution = Some(resolution);
        self
    }
    pub fn with_fps(&mut self, fps: usize) -> &mut Self {
        self.fps = Some(fps);
        self
    }
    pub fn with_length(&mut self, length: Duration) -> &mut Self {
        self.length = Some(Duration::from_secs_f64(length.as_secs_f64()));
        self
    }
    pub fn add_child(&mut self, child: Renderable) -> &mut Self {
        if self.children.is_none() {
            self.children = Some(vec![]);
        }
        self.children
            .as_mut()
            .unwrap()
            .push(Arc::new(RwLock::new(child)));
        self
    }
    pub fn build(&mut self) -> Result<Scene, SceneBuilderError> {
        let mut err = false;
        let mut report = Err(Report::new(SceneBuilderError));
        if self.children.is_none() {
            err = true;
            report = report.attach_printable("No children were added");
        }
        if self.resolution.is_none() {
            err = true;
            report = report.attach_printable("No resolution was set");
        }
        if self.fps.is_none() {
            err = true;
            report = report.attach_printable("No fps was set");
        }
        if self.length.is_none() {
            err = true;
            report = report.attach_printable("No length was set");
        }
        if err {
            return report;
        }

        Ok(Scene {
            children: std::mem::replace(&mut self.children, None).unwrap(),
            resolution: self.resolution.unwrap(),
            fps: self.fps.unwrap(),
            length: std::mem::replace(&mut self.length, None).unwrap(),
            rate_control_mode: self.rate_control_mode,
        })
    }
}

pub fn to_uv(
    top_left: Point<isize>,
    bottom_right: Point<isize>,
    point: Point<isize>,
) -> Point<f64> {
    Point::new(
        (point.x - top_left.x) as f64 / (bottom_right.x - top_left.x) as f64,
        1.0 - (point.y - top_left.y) as f64 / (bottom_right.y - top_left.y) as f64, //flip y (NOTE: This ASSUMES Image uses crt-style coordinates, may be unnecessary it it doesn't)
    )
}
