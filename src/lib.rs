#![feature(slice_pattern)]
pub mod prelude {
    pub use super::point::Point;
    pub use super::renderable::{
        closure_renderable::ClosureRenderable, Behaviour, Renderable, RenderableParams,
    };
    pub use super::resolution_consts::*;
    pub use super::scene::{Img, Scene};
    pub use error_stack::{Context, IntoReport, Report, Result, ResultExt};
    pub use error_stack_derive::ErrorStack;
    pub use image::Rgba;
}

pub mod encoding;
pub mod point;
pub mod renderable;
pub mod resolution_consts;
pub mod scene;
pub mod tools;
