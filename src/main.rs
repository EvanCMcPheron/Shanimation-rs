use error_stack::{Result, ResultExt};
use error_stack_derive::ErrorStack;

use shanimation_rs::{
    renderable::{
        renderable_image::RendreableImage, Behaviour, Renderable, RenderableParams, Rgba,
    },
    scene::{Img, Scene},
    Point,
};
use std::time::Duration;

struct BasicShader;

impl Behaviour for BasicShader {
    fn process(&mut self, params: &mut RenderableParams, _time: Duration) {
        params.position.x += 1;
        params.position.y += 2;
        params.scale.x += 0.01;
        params.scale.y -= 0.005;
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
        .with_length(Duration::from_secs(100))
        .with_resolution(Point::new(1920, 1080))
        .add_child(
            Renderable::builder()
                .with_position(Point::new(200, 150))
                .with_dimensions(Point::new(700, 500))
                .with_behaviour(Box::new(BasicShader))
                .add_child(
                    Renderable::builder()
                        .with_position(Point::new(50, 50))
                        .with_dimensions(Point::new(500, 400))
                        .with_behaviour(Box::new(
                            RendreableImage::new("TestImage.png", Box::new(|_, _, _| {})).unwrap(),
                        ))
                        .add_child(
                            Renderable::builder()
                                .with_position(Point::new(50, 0))
                                .with_dimensions(Point::new(100, 400))
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
