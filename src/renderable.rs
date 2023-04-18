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
}
