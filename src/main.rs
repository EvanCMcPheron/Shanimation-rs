use error_stack::{Result, ResultExt};
use error_stack_derive::ErrorStack;

use shanimation_rs::{
    renderable::{Behaviour, Renderable, RenderableParams, Rgba},
    resolution_consts,
    scene::{Img, Scene},
    Point,
};
use std::time::Duration;

#[derive(Clone)]
struct BasicShader;

impl Behaviour for BasicShader {
    fn process(&mut self, params: &mut RenderableParams, _time: Duration) {
        params.position.x += _time.as_secs_f64().cos() * 0.1;
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

#[derive(Clone)]
struct WhiteRect;

impl Behaviour for WhiteRect {
    fn process(&mut self, _renderable: &mut RenderableParams, _time: Duration) {}
    fn get_pixel(&self, _current_frame: &Img, _uv_coords: Point<f64>, _time: Duration) -> Rgba<u8> {
        Rgba([255, 255, 255, 120])
    }
}

#[derive(Clone)]
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
        .with_length(Duration::from_secs(10))
        .with_resolution(Point::new(1920, 1080))
        .with_fps(60)
        .add_child(
            Renderable::builder()
                .with_position(Point::new(0.1, 0.1))
                .with_size(Point::new(0.35, 0.5))
                .with_behaviour(Box::new(BasicShader))
                .add_child(
                    Renderable::builder()
                        .with_position(Point::new(0.05, 0.02))
                        .with_size(Point::new(0.26, 0.4))
                        .with_behaviour(Box::new(WhiteRect))
                        .add_child(
                            Renderable::builder()
                                .with_position(Point::new(0.05, 0.0))
                                .with_size(Point::new(0.05, 0.4))
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
