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

use super::renderable::*;
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
}

#[derive(ErrorStack, Debug)]
#[error_message("An error occured while rendering a scene")]
pub enum SceneRenderingError {
    FileWritingError,
    FrameRenderingError,
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
        Ok(())
    }
    fn render_frame(&self, frame_indx: usize, time: Duration) -> Result<(), SceneRenderingError> {
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
            todo!()
        }
        //write image buffer to file
        todo!()
    }
    /*
    fn post_process_frame() -> Result<(), SceneRenderingError> {
        In future, potentially add an optional post-processing feild to scene, which takes an takes the final image buffer and is free to operate on it any way it wants.
        That would be run in this fn, which would be run directly after the 'render_frame()' fn.
    }
     */
    fn generate_frame_dictionary() -> Result<(), SceneRenderingError> {
        //Create and save a frame dictionary
        todo!()
    }
    fn compile_video() -> Result<(), SceneRenderingError> {
        //Use this command (add formatting)
        //ffmpeg -reinit_filter 0 -f concat -safe 0 -i "ffmpeg.Txt" -vf "scale=1280:720:force_original_aspect_ratio=decrease:eval=frame,pad=1280:720:-1:-1:color=black:eval=frame,settb=AVTB,format=yuv420p" -r 15 output.mp4
        //print outputs to stdout, maybe in the future add formatting?
        todo!()
    }
    pub fn render(&self) -> Result<(), SceneRenderingError> {
        todo!()
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
