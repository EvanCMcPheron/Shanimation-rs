use error_stack::{IntoReport, Report, Result, ResultExt};
use error_stack_derive::ErrorStack;

use image::Rgba;
use std::fs::DirBuilder;
use std::io::Write;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use super::encoding::rgba_to_yuv;
use super::frame_dictionary::FrameDict;
use super::renderable::*;
use super::Point;
use super::RateControlMode;
use image::RgbaImage;
use openh264::encoder::{Encoder, EncoderConfig};

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
}

pub struct Scene {
    children: Vec<Arc<RwLock<Renderable>>>,
    resolution: Point<usize>,
    fps: usize,
    length: Duration,
    output_filename: PathBuf,
    rate_control_mode: RateControlMode,
}

impl Scene {
    pub fn builder() -> SceneBuilder {
        SceneBuilder {
            children: Some(vec![]),
            resolution: Some(Point::new(1280, 720)),
            fps: Some(30),
            length: None,
            output_filename: Some(PathBuf::from("output.mp4")),
            rate_control_mode: RateControlMode::Off,
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
        //for each frame, run render frame
        for (frame_indx, time) in (0..max_frames)
            .map(|i| Duration::from_secs_f64(i as f64 * seconds_per_frame))
            .enumerate()
        {
            //TODO: Potentially introduce a threadpool here to 'concurrently' render all frames
            encoder
                .encode(&rgba_to_yuv(
                    self.render_frame(frame_indx, time)
                        .change_context(SceneRenderingError::FrameRenderingError)
                        .attach_printable_lazy(|| {
                            format!(
                                "Failed to render frame {} at time {} seconds",
                                frame_indx,
                                time.as_secs_f64()
                            )
                        })?,
                ))
                .into_report()
                .change_context(SceneRenderingError::EncodingError)
                .attach_printable_lazy(|| "Failed to encode frame")?
                .write_vec(&mut video_bytes);
        }
        Ok(video_bytes)
    }
    fn render_frame(
        &self,
        frame_indx: usize,
        time: Duration,
    ) -> Result<RgbaImage, SceneRenderingError> {
        use std::cmp::{max, min};
        //create an empty rgba image buffer
        let mut img_buffer = Img::new(self.resolution);
        //'recursively' iterate through all children of the scene, and their children, (run from top down)
        let mut stack: Vec<(Point<isize>, Point<f64>, Arc<RwLock<Renderable>>)> = vec![]; //(offset, child)
        self.children
            .iter()
            .map(Clone::clone)
            .map(|c| (Point::new(0, 0), Point::new(1.0, 1.0), c))
            .for_each(|v| stack.push(v));
        //for each child, run their run their behaviour's process, then for every pixel, run their get_pixel (THIS CAN EASILY BE PARRELLELIZED) and overide the pixel on the main image buffer'
        while let Some((residual_offset, residual_scale, child)) = stack.pop() {
            let mut child = child.write().unwrap();

            // This scale should be multiplied against the dimensions when calculating the bottom right point (and also get passed to children), but only residual scale should be applied to the top left point.
            let next_scale = Point::new(
                residual_scale.x * child.params.scale.x,
                residual_scale.y * child.params.scale.y,
            );

            //This should be passed down to children AND used for coordinate calculations
            let next_offset = Point::new(
                residual_offset.x + (child.params.position.x as f64 * residual_scale.x) as isize,
                residual_offset.y + (child.params.position.y as f64 * residual_scale.y) as isize,
            );

            child
                .get_children()
                .iter()
                .map(Clone::clone)
                .map(|c| (next_offset, next_scale, c))
                .for_each(|c| stack.push(c));
            child.run_behaviour(time);
            //For every pixel within the bounds of the shader, run the get_pixel fn and overide the pixel on the main image buffer
            let up_left_unchecked = Point::new(next_offset.x, next_offset.y);
            let down_right_unchecked = Point::new(
                up_left_unchecked.x + ((child.params.dimensions.x as f64 * next_scale.x) as isize),
                up_left_unchecked.y + ((child.params.dimensions.y as f64 * next_scale.y) as isize),
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
                    let uv = to_uv(up_left_unchecked, down_right_unchecked, p);
                    let color = child.run_shader(&old_image, uv, time);
                    let c = Point::new(p.x as usize, p.y as usize);
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
                    img_buffer.set_pixel(c, new_color);
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
        let mut output_filename = "./output/scene_0.h264".to_owned();
        let mut i = 1;
        while Path::new(&output_filename).exists() {
            output_filename = format!("./output/scene_{}.h264", i);
            i += 1;
        }
        //Use this command (add formatting)
        //self.run_ffmpeg_cmd(output_filename)
        let file = std::fs::File::create(&output_filename);
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

        let mut mp4_path = path.clone();
        mp4_path.set_extension("mp4");

        Command::new("ffmpeg")
            .current_dir(glob_path)
            .arg("-i")
            .arg(format!("{}", path.display()))
            .arg(format!("{}", mp4_path.display()))
            .spawn()
            .into_report()
            .change_context(SceneRenderingError::FFMPEGError)
            .attach_printable_lazy(|| "Failed to convert video file")?;

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
    output_filename: Option<PathBuf>,
    rate_control_mode: RateControlMode,
}

impl SceneBuilder {
    pub fn with_rate_control_mode(&mut self, mode: RateControlMode) -> &mut Self {
        self.rate_control_mode = mode;
        self
    }
    pub fn with_resolution(&mut self, resolution: Point<usize>) -> &mut Self {
        self.resolution = Some(resolution);
        self
    }
    pub fn with_fps(&mut self, fps: usize) -> &mut Self {
        self.fps = Some(fps);
        self
    }
    pub fn with_length(&mut self, length: Duration) -> &mut Self {
        self.length = Some(length);
        self
    }
    pub fn with_output_filename<P: AsRef<Path> + ?Sized>(
        &mut self,
        output_filename: &P,
    ) -> &mut Self {
        self.output_filename = Some(output_filename.as_ref().to_path_buf());
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
        if self.output_filename.is_none() {
            err = true;
            report = report.attach_printable("No output filename was set");
        }
        if err {
            return report;
        }

        Ok(Scene {
            children: std::mem::replace(&mut self.children, None).unwrap(),
            resolution: self.resolution.unwrap(),
            fps: self.fps.unwrap(),
            length: std::mem::replace(&mut self.length, None).unwrap(),
            output_filename: std::mem::replace(&mut self.output_filename, None).unwrap(),
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
