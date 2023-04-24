use error_stack::{Result, ResultExt};
use error_stack_derive::ErrorStack;

use shanimation_rs::prelude::*;
use std::time::Duration;



#[derive(Debug, ErrorStack)]
#[error_message("Error occured in main fn")]
pub struct MainError;

fn main() -> Result<(), MainError> {
    let main_renderable = Renderable::builder()
        .with_position(0.1, 0.1)
        .with_size(0.3, 0.3)
        .with_behaviour(Box::new(ClosureRenderable {
            data: (),
            process: |data, params, time| {
                params.position.x += time.as_secs_f64().cos() * 0.02;
            },
            shader: |data, frame, uv, time| -> Rgba<u8> {
                let p = uv.map_both(|v| (v * 255.0) as u8);
                Rgba([255, p.x, p.y, 255])
            },
        }))
        .build()
        .unwrap();

    Scene::builder()
        .with_length(Duration::from_secs(10))
        .with_resolution(RESOLUTION_1080P)
        .with_fps(60)
        .add_child(main_renderable)
        .build()
        .change_context(MainError)
        .attach_printable_lazy(|| "Failed to create scene")?
        .render()
        .change_context(MainError)
}
