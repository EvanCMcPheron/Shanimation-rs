#![feature(slice_pattern)]
pub mod prelude {
    pub use super::point::Point;
    pub use error_stack::{Context, IntoReport, Report, Result, ResultExt};
    pub use error_stack_derive::ErrorStack;
    pub use super::scene::{Scene, Img};
    pub use image::Rgba;
    pub use super::renderable::{Renderable, Behaviour, closure_renderable::ClosureRenderable, RenderableParams};
    pub use super::resolution_consts::*;
}

pub mod encoding;
mod frame_dictionary;
pub mod point;
pub mod renderable;
pub mod resolution_consts;
pub mod scene;
pub mod tools;
