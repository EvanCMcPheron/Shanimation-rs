use error_stack::{Context, IntoReport, Report, Result, ResultExt};
use error_stack_derive::ErrorStack;
use shanimation::{
    frame_dictionary::FrameDict,
    scene::Scene,
    renderable::{
        Renderable,
        Behaviour,
        FragShader,
        Rgba,
    },
    Point
};
use std::time::Duration;

struct MyBehaviour;

impl Behaviour for MyBehaviour {
    fn process(&mut self, _: Box<&mut dyn FragShader>, _: Duration) {}
}

struct MyFragShader;

impl FragShader for MyFragShader {
    fn get_pixel(&self, uv_coords: Point<f32>, time: Duration) -> Rgba<u8> {
        let v = (255 as f64 * time.as_secs_f64()) as u8;
        Rgba([v, v, v, 255])
    }
}

#[derive(Debug, ErrorStack)]
#[error_message("Error occured in main fn")]
pub enum MainError {
    FrameDictCreation,
    SceneCreation
}

fn main() -> Result<(), MainError> {
    FrameDict { frame_count: 30 }
        .save()
        .change_context(MainError::FrameDictCreation)?;

    Scene::builder()
        .with_length(Duration::from_secs(1))
        .add_child(
            Renderable::builder()
                .with_dimensions(Point::new(500,500))
                .with_behaviour(Box::new(MyBehaviour))
                .with_shader(Box::new(MyFragShader))
                .build()
                .change_context(MainError::SceneCreation)
                .attach_printable_lazy(|| "Failed to create renderable")?
        )
        .build()
        .change_context(MainError::SceneCreation)
        .attach_printable_lazy(|| "Failed to create scene")?;

    Ok(())
}
