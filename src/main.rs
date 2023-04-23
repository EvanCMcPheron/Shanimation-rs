use error_stack::{Result, ResultExt};
use error_stack_derive::ErrorStack;

use shanimation_rs::{
    renderable::{
        renderable_image::RendreableImage, Behaviour, Renderable, RenderableParams, Rgba,
    },
    scene::{Img, Scene},
    Point,
    resolution_consts::*,
};
use std::time::Duration;

struct BasicShader;

impl Behaviour for BasicShader {
    fn process(&mut self, params: &mut RenderableParams, _time: Duration) {
        params.position.x += (_time.as_secs_f64()).cos() * 0.01;
    }
    fn get_pixel(&self, _current_frame: &Img, uv_coords: Point<f64>, _time: Duration) -> Rgba<u8> {
        Rgba([
            0,
            (255.0 * uv_coords.x) as u8,
            (255.0 * uv_coords.y) as u8,
            255,
        ])
    }
}

struct WhiteRect;

impl Behaviour for WhiteRect {
    fn process(&mut self, _renderable: &mut RenderableParams, _time: Duration) {}
    fn get_pixel(&self, _current_frame: &Img, _uv_coords: Point<f64>, _time: Duration) -> Rgba<u8> {
        Rgba([255, 255, 255, 120])
    }
}

struct RedRect;

impl Behaviour for RedRect {
    fn process(&mut self, _renderable: &mut RenderableParams, _time: Duration) {}
    fn get_pixel(&self, _current_frame: &Img, _uv_coords: Point<f64>, _time: Duration) -> Rgba<u8> {
        Rgba([255, 0, 0, 120])
    }
}

#[derive(Debug, ErrorStack)]
#[error_message("Error occured in main fn")]
pub enum MainError {
    FrameDictCreation,
    SceneCreation,
    SceneRendering,
}

fn main() -> Result<(), MainError> {
    Scene::builder()
    .with_length(Duration::from_secs(20))
    .with_resolution(RESOLUTION_1080P)
    .with_fps(60)
    .add_child(
        Renderable::builder()
            .with_position(Point::new(0.2, 0.1))
            .with_size(Point::new(0.4, 0.4))
            .with_behaviour(Box::new(BasicShader))
            .add_child(
                Renderable::builder()
                    .with_position(Point::new(0.05, 0.05))
                    .with_size(Point::new(0.25, 0.4))
                    .with_behaviour(Box::new(
                        RendreableImage::new("TestImage.png", Box::new(|_, _, _| {})).unwrap(),
                    ))
                    .add_child(
                        Renderable::builder()
                            .with_position(Point::new(0.05, 0.0))
                            .with_size(Point::new(0.05, 0.2))
                            .with_behaviour(Box::new(RedRect))
                            .build()
                            .unwrap(),
                    )
                    .build()
                    .unwrap(),
            )
            .build()
            .change_context(MainError::SceneCreation)
            .attach_printable_lazy(|| "Failed to create renderable")?,
    )
    .build()
    .change_context(MainError::SceneCreation)
    .attach_printable_lazy(|| "Failed to create scene")?
    .render()
    .change_context(MainError::SceneRendering)?;
    Ok(())
}
