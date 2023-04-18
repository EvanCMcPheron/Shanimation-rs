use error_stack::{Context, IntoReport, Report, Result, ResultExt};
use error_stack_derive::ErrorStack;
use image::Rgba;
pub use imageproc::point::Point;
use std::fs::{DirBuilder, File};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub trait FragShader {
    fn get_pixel(&self, uv_coords: Point<f32>, time: Duration) -> Rgba<u8>; //Not intended to mutate any state in this method
}

pub trait Behaviour {
    fn process(&mut self, shader: Box<&mut dyn FragShader>, time: Duration); //Would be nice to figure out how to use an associated type for the shader, but that would require a generic in the Renderable declaration, which is not possible due to the recursive tree-like structure
}

pub struct Renderable {
    children: Vec<Arc<RwLock<Renderable>>>,
    pub position: Point<isize>,
    pub dimensions: Point<usize>,
    shader: Box<dyn FragShader>,
    behaviour: Box<dyn Behaviour>,
}

impl Renderable {
    fn add_child(&mut self, child: Arc<RwLock<Renderable>>) {
        self.children.push(child);
    }
    fn add_child_simple(&mut self, child: Renderable) -> Arc<RwLock<Renderable>> {
        //! Creates an Arc<RwLock<Renderable>> from child, clones it and adds it to children, then returns the Arc<RwLock<Renderable>>
        let arc = Arc::new(RwLock::new(child));
        self.add_child(arc.clone());
        arc
    }
    fn get_children(&self) -> &Vec<Arc<RwLock<Renderable>>> {
        &self.children
    }
    fn get_children_mut(&mut self) -> &mut Vec<Arc<RwLock<Renderable>>> {
        &mut self.children
    }
    fn run_shader(&mut self, uv_coords: Point<f32>, time: Duration) -> Rgba<u8> {
        self.shader.get_pixel(uv_coords, time)
    }
    fn run_behaviour(&mut self, time: Duration) {
        self.behaviour.process(Box::new(self.shader.as_mut()), time);
    }
    fn builder() -> RenderableBuilder {
        RenderableBuilder { children: vec![], position: None, dimensions: None, shader: None, behaviour: None }
    }
}

#[derive(ErrorStack, Debug, Clone)]
#[error_message("Not all builder requirements were fullfilled")]
pub struct RenderableBuilderError;

pub struct RenderableBuilder {
    children: Vec<Arc<RwLock<Renderable>>>,
    position: Option<Point<isize>>,
    dimensions: Option<Point<usize>>,
    shader: Option<Box<dyn FragShader>>,
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
    pub fn with_shader(&mut self, shader: Box<dyn FragShader>) -> &mut Self {
        self.shader = Some(shader);
        self
    }
    pub fn with_behaviour<'b>(&mut self, behaviour: Box<dyn Behaviour>) -> &mut Self {
        self.behaviour = Some(behaviour);
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
        if self.shader.is_none() {
            err = true;
            report = report.attach_printable("No shader was set");
        }
        if self.behaviour.is_none() {
            err = true;
            report = report.attach_printable("No behaviour was set");
        }
        if err {
            return report;
        }

        struct DummyShader;
        impl FragShader for DummyShader {
            fn get_pixel(&self, _: Point<f32>, _: Duration) -> Rgba<u8> {
                Rgba([0, 0, 0, 0])
            }
        }
        struct DummyBehaviour;
        impl Behaviour for DummyBehaviour {
            fn process(&mut self, _: Box<&mut dyn FragShader>, _: Duration) {}
            
        }        

        Ok(Renderable {
            children: std::mem::replace(&mut self.children, vec![]),
            position: self.position.unwrap(),
            dimensions: self.dimensions.unwrap(),
            shader: std::mem::replace(&mut self.shader, Some(Box::new(DummyShader))).unwrap(),
            behaviour: std::mem::replace(&mut self.behaviour, Some(Box::new(DummyBehaviour))).unwrap(),
        })
    }
}
