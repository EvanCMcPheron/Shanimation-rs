use error_stack::{Result, ResultExt};
use error_stack_derive::ErrorStack;

use shanimation_rs::{prelude::*, scene};
use std::time::Duration;



#[derive(Debug, ErrorStack)]
#[error_message("Error occured in main fn")]
pub struct MainError;

fn main() -> Result<(), MainError> {
    let main_renderable = Renderable::builder()
        .with_position(0.2, 0.0)
        .with_size(0.3, 0.3)
        .with_rotation(2.0)
        .with_behaviour(Box::new(ClosureRenderable {
            data: (),
            process: |data, params, time, scene, abs_position| {
                params.rotation = time.as_secs_f64() * 2.0;
            },
            shader: |data, frame, uv, time, abs_position| -> Rgba<u8> {
                let p = uv.map_both(|v| (v * 255.0) as u8);
                Rgba([255, p.x, p.y, 255])
            },
        }))
        .build()
        .unwrap();

    Scene::builder()
        .with_length(Duration::from_secs(10))
        .with_resolution(RESOLUTION_720P)
        .with_fps(30)
        .add_child(main_renderable)
        .build()
        .change_context(MainError)
        .attach_printable_lazy(|| "Failed to create scene")?
        .render()
        .change_context(MainError)
}
