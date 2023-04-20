use super::scene::Img;
pub use super::Point;
use error_stack::{Context, IntoReport, Report, Result, ResultExt};
use error_stack_derive::ErrorStack;
pub use image::Rgba;
use std::fs::{DirBuilder, File};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub mod rendered_image;

pub trait Behaviour {
    fn process(&mut self, renderable: &mut RenderableParams, time: Duration);
    fn get_pixel(&self, current_frame: &Img, uv_coords: Point<f64>, time: Duration) -> Rgba<u8>; //Not intended to mutate any state in this method
}

pub struct Renderable {
    pub params: RenderableParams,
    behaviour: Box<dyn Behaviour>,
}

pub struct RenderableParams {
    children: Vec<Arc<RwLock<Renderable>>>,
    pub scale: Point<f64>,
    pub position: Point<isize>,
    pub dimensions: Point<usize>,
}

impl RenderableParams {
    pub fn add_child(&mut self, child: Arc<RwLock<Renderable>>) {
        self.children.push(child);
    }
    pub fn add_child_simple(&mut self, child: Renderable) -> Arc<RwLock<Renderable>> {
        //! Creates an Arc<RwLock<Renderable>> from child, clones it and adds it to children, then returns the Arc<RwLock<Renderable>>
        let arc = Arc::new(RwLock::new(child));
        self.add_child(arc.clone());
        arc
    }
    pub fn get_children(&self) -> &Vec<Arc<RwLock<Renderable>>> {
        &self.children
    }
    pub fn get_children_mut(&mut self) -> &mut Vec<Arc<RwLock<Renderable>>> {
        &mut self.children
    }
}

impl Renderable {
    pub fn add_child(&mut self, child: Arc<RwLock<Renderable>>) {
        self.params.add_child(child);
    }
    pub fn add_child_simple(&mut self, child: Renderable) -> Arc<RwLock<Renderable>> {
        //! Creates an Arc<RwLock<Renderable>> from child, clones it and adds it to children, then returns the Arc<RwLock<Renderable>>
        self.params.add_child_simple(child)
    }
    pub fn get_children(&self) -> &Vec<Arc<RwLock<Renderable>>> {
        self.params.get_children()
    }
    pub fn get_children_mut(&mut self) -> &mut Vec<Arc<RwLock<Renderable>>> {
        self.params.get_children_mut()
    }
    pub fn run_shader(
        &mut self,
        current_frame: &Img,
        uv_coords: Point<f64>,
        time: Duration,
    ) -> Rgba<u8> {
        self.behaviour.get_pixel(current_frame, uv_coords, time)
    }
    pub fn run_behaviour(&mut self, time: Duration) {
        self.behaviour.process(&mut self.params, time);
    }
    pub fn builder() -> RenderableBuilder {
        RenderableBuilder {
            scale: Point::new(1.0, 1.0),
            children: vec![],
            position: Some(Point::new(0, 0)),
            dimensions: Some(Point::new(1280, 720)),
            behaviour: None,
        }
    }
}

#[derive(ErrorStack, Debug, Clone)]
#[error_message("Not all builder requirements were fullfilled")]
pub struct RenderableBuilderError;

pub struct RenderableBuilder {
    children: Vec<Arc<RwLock<Renderable>>>,
    position: Option<Point<isize>>,
    scale: Point<f64>,
    dimensions: Option<Point<usize>>,
    behaviour: Option<Box<dyn Behaviour>>,
}

impl RenderableBuilder {
    pub fn add_child(&mut self, child: Renderable) -> &mut Self {
        self.children.push(Arc::new(RwLock::new(child)));
        self
    }
    pub fn with_position(&mut self, position: Point<isize>) -> &mut Self {
        self.position = Some(position);
        self
    }
    pub fn with_dimensions(&mut self, dimensions: Point<usize>) -> &mut Self {
        self.dimensions = Some(dimensions);
        self
    }
    pub fn with_behaviour<'b>(&mut self, behaviour: Box<dyn Behaviour>) -> &mut Self {
        self.behaviour = Some(behaviour);
        self
    }
    pub fn with_scale(&mut self, scale: Point<f64>) -> &mut Self {
        self.scale = scale;
        self
    }
    pub fn build(&mut self) -> Result<Renderable, RenderableBuilderError> {
        let mut err = false;
        let mut report = Err(Report::new(RenderableBuilderError));
        if self.position.is_none() {
            err = true;
            report = report.attach_printable("No position was set");
        }
        if self.dimensions.is_none() {
            err = true;
            report = report.attach_printable("No dimensions were set");
        }
        if self.behaviour.is_none() {
            err = true;
            report = report.attach_printable("No behaviour was set");
        }
        if err {
            return report;
        }

        struct DummyBehaviour;
        impl Behaviour for DummyBehaviour {
            fn process(&mut self, _: &mut RenderableParams, _: Duration) {}
            fn get_pixel(
                &self,
                current_frame: &Img,
                uv_coords: Point<f64>,
                time: Duration,
            ) -> Rgba<u8> {
                Rgba([0, 0, 0, 0])
            }
        }

        Ok(Renderable {
            params: RenderableParams {
                children: std::mem::replace(&mut self.children, vec![]),
                scale: self.scale,
                position: self.position.unwrap(),
                dimensions: self.dimensions.unwrap(),
            },
            behaviour: std::mem::replace(&mut self.behaviour, Some(Box::new(DummyBehaviour)))
                .unwrap(),
        })
    }
}
