use error_stack::{Context, IntoReport, Report, Result, ResultExt};
use error_stack_derive::ErrorStack;
use image::Rgba;
pub use imageproc::point::Point;
use std::fs::{DirBuilder, File};
use std::io::Write;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use image::ImageFormat;
use std::process::Command;

use super::renderable::*;
use super::frame_dictionary::FrameDict;
use image::RgbaImage;

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
    FrameDictCreationError,
    FFMPEGError,
}

pub struct Scene {
    children: Vec<Arc<RwLock<Renderable>>>,
    resolution: Point<usize>,
    fps: usize,
    length: Duration,
    output_filename: PathBuf,
}

impl Scene {
    pub fn builder() -> SceneBuilder {
        SceneBuilder {
            children: Some(vec![]),
            resolution: Some(Point::new(1280, 720)),
            fps: Some(30),
            length: None,
            output_filename: Some(PathBuf::from("output.mp4")),
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
    fn render_frames(&self) -> Result<(), SceneRenderingError> {
        //delete frame directory
        if Path::new("./frames").exists() {
            std::fs::remove_dir_all("./frames")
                .into_report()
                .change_context(SceneRenderingError::FileWritingError)
                .attach_printable("Failed to delete frame directory")?;
        }
        //create frame directory
        DirBuilder::new()
            .recursive(true)
            .create("./frames")
            .into_report()
            .change_context(SceneRenderingError::FileWritingError)
            .attach_printable("Failed to create frame directory")?;
        //figure out frame count, with matching duration to send to behaviour and shader
        let max_frames = self.length.as_secs() as usize * self.fps;
        let seconds_per_frame = 1.0 / self.fps as f64;
        //for each frame, run render frame
        for (frame_indx, time) in (0..max_frames)
            .map(|i| Duration::from_secs_f64(i as f64 * seconds_per_frame))
            .enumerate()
        {
            //TODO: Potentially introduce a threadpool here to 'concurrently' render all frames
            self.render_frame(frame_indx, time)
                .change_context(SceneRenderingError::FrameRenderingError)
                .attach_printable_lazy(|| {
                    format!(
                        "Failed to render frame {} at time {} seconds",
                        frame_indx,
                        time.as_secs_f64()
                    )
                })?;
        }
        self.generate_frame_dictionary(max_frames)
    }
    fn render_frame(&self, frame_indx: usize, time: Duration) -> Result<(), SceneRenderingError> {
        use std::cmp::{min, max};

        //create an empty rgba image buffer
        let mut img_buffer = Img::new(self.resolution);
        //'recursively' iterate through all children of the scene, and their children, (run from top down)
        let mut stack = vec![];
        self.children.iter().map(Clone::clone).for_each(|c| stack.push(c));
        //for each child, run their run their behaviour's process, then for every pixel, run their get_pixel (THIS CAN EASILY BE PARRELLELIZED) and overide the pixel on the main image buffer'
        while let Some(child) = stack.pop() {
            let mut child = child.write().unwrap();
            child.get_children().iter().map(Clone::clone).for_each(|c| stack.push(c));
            child.run_behaviour(time);
            //For every pixel within the bounds of the shader, run the get_pixel fn and overide the pixel on the main image buffer
            let up_left = Point::new(
                max(0, child.position.x),
                max(0, child.position.y),
            );
            let down_right = Point::new(
                min(self.resolution.x as isize, child.position.x + child.dimensions.x as isize),
                min(self.resolution.y as isize, child.position.y + child.dimensions.y as isize),
            );
            ((up_left.y)..=(down_right.y))
                .map(|y| {
                    ((up_left.x)..=(down_right.x))
                        .map(|x| {
                            Point::new(x, y)
                        })
                        .collect::<Vec<Point<isize>>>() // *may* cause bottleneck
                })
                .flatten()
                .for_each(|p| {
                    let uv = to_uv(up_left, down_right, p);
                    let color = child.run_shader(&img_buffer, uv, time);
                    let c = Point::new(p.x as usize, p.y as usize);
                    let current_color = img_buffer.get_pixel(c);
                    let a = color.0[3];
                    let mixer = |i: usize| (color.0[i] as f32 / 255.0 * a as f32 + current_color.0[i] as f32 / 255.0 * (255.0 - a as f32)) as u8;
                    let a_mixer = || a + current_color.0[3] * (255 - a);    //Not sure if this is how alpha mixing SHOULD work, but it seems right?
                    let new_color = Rgba([
                        mixer(0),
                        mixer(1),
                        mixer(2),
                        a_mixer()
                    ]);
                    img_buffer.set_pixel(c, new_color);
                })
            
        }
        //write image buffer to file
        img_buffer.image.save(format!("./frames/{}.png", frame_indx))
            .into_report()
            .change_context(SceneRenderingError::FileWritingError)
            .attach_printable_lazy(|| format!("Failed to write frame {} to file", frame_indx))?;
        Ok(())
    }
    /*
    fn post_process_frame() -> Result<(), SceneRenderingError> {
        In future, potentially add an optional post-processing feild to scene, which takes an takes the final image buffer and is free to operate on it any way it wants.
        That would be run in this fn, which would be run directly after the 'render_frame()' fn.
    }
     */
    fn generate_frame_dictionary(&self, frame_count: usize) -> Result<(), SceneRenderingError> {
        //Create and save a frame dictionary
        FrameDict {frame_count}.save()
            .change_context(SceneRenderingError::FrameDictCreationError)
            .attach_printable_lazy(|| "Failed to create frame dictionary")
    }
    fn compile_video(&self) -> Result<(), SceneRenderingError> {
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
        //Use this command (add formatting)
        self.run_ffmpeg_cmd(output_filename)
    }
    fn run_ffmpeg_cmd(&self, path: String) -> Result<(), SceneRenderingError> {
        //ffmpeg -reinit_filter 0 -f concat -safe 0 -i "frames/dict.txt" -vf "scale=1280:720:force_original_aspect_ratio=decrease:eval=frame,pad=1280:720:-1:-1:color=black:eval=frame,settb=AVTB,setpts=0.033333333*N/TB,format=yuv420p" -r 30 -movflags +faststart output.mp4
        let ffmpeg = Command::new("ffmpeg")
            .args(format!("-reinit_filter 0 -f concat -safe 0 -i \"frames/dict.txt\" -vf \"scale={}:{}:force_original_aspect_ratio=decrease:eval=frame,pad={}:{}:-1:-1:color=black:eval=frame,settb=AVTB,setpts={}*N/TB,format=yuv420p\" -r {} -movflags +faststart {path}", self.resolution.x, self.resolution.y, self.resolution.x, self.resolution.y, 1.0 / self.fps as f32, self.fps).split_ascii_whitespace())
            .spawn()
            .into_report()
            .change_context(SceneRenderingError::FFMPEGError)
            .attach_printable_lazy(|| "Failed to concatinate frames into video through ffmpeg")?;
        Ok(())
    }
    fn delete_frames_directory(&self) -> Result<(), SceneRenderingError> {
        todo!()
    }
    pub fn render(&self) -> Result<(), SceneRenderingError> {
        self.render_frames()
            .attach_printable_lazy(|| "Failed to render frames")?;
        self.compile_video()
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
        })
    }
}

pub fn to_uv(top_left: Point<isize>, bottom_right: Point<isize>, point: Point<isize>) -> Point<f64> {
    Point::new(
        (point.x - top_left.x) as f64 / (bottom_right.x - top_left.x) as f64, 
        1.0 - (point.y - top_left.y) as f64 / (bottom_right.y - top_left.y) as f64  //flip y (NOTE: This ASSUMES Image uses crt-style coordinates, may be unnecessary it it doesn't)
    )
}
