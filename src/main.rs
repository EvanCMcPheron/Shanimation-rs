use error_stack::{Result, ResultExt};
use error_stack_derive::ErrorStack;

use shanimation_rs::{prelude::*, scene, tools::curves::{
    chainable_curves::*,
    Curve,
}};
use std::time::Duration;

#[derive(Debug, ErrorStack)]
#[error_message("Error occured in main fn")]
pub struct MainError;

fn main() -> Result<(), MainError> {
    let main_renderable = Renderable::builder()
        .with_position(0.2, 0.1)
        .with_size(0.3, 0.3)
        .with_rotation(2.0)
        .with_behaviour(Box::new(ClosureRenderable {
            data: (
                SmoothCurve(vec![Point::new(1.0, 0.0), Point::new(2.0, 0.4), Point::new(3.0, 0.4), Point::new(5.0, 0.1)]),
                SmoothCurve(vec![Point::new(0.3, 0.01), Point::new(1.0, 0.1), Point::new(2.0, 0.1), Point::new(2.7, 0.01)]),
                0.01,
            ),
            process: |data, params, time, scene, abs_position| {
                params.rotation += data.2;
                params.position.x = data.0.get_value(time.as_secs_f64());
                data.2 = data.1.get_value(time.as_secs_f64());
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
