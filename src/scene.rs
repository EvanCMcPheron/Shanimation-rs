use error_stack::{Context, IntoReport, Report, Result, ResultExt};
use error_stack_derive::ErrorStack;
use image::Rgba;
pub use imageproc::point::Point;
use std::fs::{DirBuilder, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use super::renderable::*;

pub struct Scene {
    children: Vec<Arc<RwLock<Renderable>>>,
    resolution: Point<usize>,
    fps: usize,
    length: Duration,
    output_filename: PathBuf,
}

impl Scene {
    pub fn builder() -> SceneBuilder {
        SceneBuilder { children: Some(vec![]), resolution: Some(Point::new(1280,720)), fps: Some(30), length: None, output_filename: Some(PathBuf::from("output.mp4")) }
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
    pub fn with_output_filename<P: AsRef<Path> + ?Sized>(&mut self, output_filename: &P) -> &mut Self {
        self.output_filename = Some(output_filename.as_ref().to_path_buf());
        self
    }
    pub fn add_child(&mut self, child: Renderable) -> &mut Self {
        if self.children.is_none() {
            self.children = Some(vec![]);
        }
        self.children.as_mut().unwrap().push(Arc::new(RwLock::new(child)));
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