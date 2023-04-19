use error_stack::{Context, IntoReport, Report, Result, ResultExt};
use error_stack_derive::ErrorStack;
use image::Rgb;
use shanimation_rs::{
    frame_dictionary::FrameDict,
    renderable::{Behaviour, Renderable, RenderableParams, Rgba},
    scene::{Img, Scene},
    Point,
};
use std::time::Duration;

struct MyBehaviour {
    fpos: Point<f64>,
    radius: f64,
}

impl Behaviour for MyBehaviour {
    fn process(
        &mut self,
        params: &mut RenderableParams,
        time: Duration,
    ) {
        params.position = Point::new(self.fpos.x as isize, self.fpos.y as isize);
        self.fpos.x += 9.141592;
        self.radius -= 0.01;
    }
    fn get_pixel(&self, current_frame: &Img, uv_coords: Point<f64>, time: Duration) -> Rgba<u8> {
        let c = Point::new(uv_coords.x * 2.0 - 1.0, uv_coords.y * 2.0 - 1.0);
        let r = (c.x * c.x + c.y * c.y).sqrt() < self.radius;
        Rgba([0, (255.0 * 2.0 * c.x) as u8, (255.0 * 2.0* c.y) as u8, r as u8 * 255])
    }
}

struct EmptyBehaviour;

impl Behaviour for EmptyBehaviour {
    fn process(
            &mut self,
            renderable: &mut RenderableParams,
            time: Duration,
        ) {
        
    }
    fn get_pixel(&self, current_frame: &Img, uv_coords: Point<f64>, time: Duration) -> Rgba<u8> {
        Rgba([255, 0, 0, 10])
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
        .with_length(Duration::from_secs(1))
        .add_child(
            Renderable::builder()
                .with_position(Point::new(400, 100))
                .with_dimensions(Point::new(100, 1000))
                .with_behaviour(Box::new(EmptyBehaviour))
                .build()
                .change_context(MainError::SceneCreation)
                .attach_printable_lazy(|| "Failed to create renderable")?,
        )
        .add_child(
            Renderable::builder()
                .with_position(Point::new(200, 150))
                .with_dimensions(Point::new(300, 300))
                .with_behaviour(Box::new(MyBehaviour {
                    fpos: Point::new(200.0, 150.0),
                    radius: 0.5,
                }))
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
