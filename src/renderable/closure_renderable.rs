use crate::prelude::*;
use std::time::Duration;

/// Ex:
/// ```
/// Renderable::builder()
///     .with_position(Point::new(0.1, 0.1))
///     .with_size(Point::new(0.3, 0.3))
///     .with_behaviour(Box::new(ClosureRenderable {
///         data: (),
///         process: |data, params, time| {
///             params.position.x += time.as_secs_f64().cos() * 0.02;
///         },
///         shader: |data, frame, uv, time| -> Rgba<u8> {
///             let p = uv.map_both(|v| (v * 255.0) as u8);
///             Rgba([255, p.x, p.y, 255])
///         },
///     }))
///     .build()
///     .unwrap();
///```
#[derive(Clone)]
pub struct ClosureRenderable<T, P, S>
where  
    T: Clone + Send + Sync,
    P: Fn(&mut T, &mut RenderableParams, Duration) + Clone + Send + Sync,
    S: Fn(&T, &Img, Point<f64>, Duration) -> Rgba<u8> + Clone + Send + Sync,
{
    pub data: T,
    pub process: P,
    pub shader: S,
}

impl<T, P, S> Behaviour for ClosureRenderable<T, P, S>
where  
    T: Clone + Send + Sync,
    P: Fn(&mut T, &mut RenderableParams, Duration) + Clone + Send + Sync,
    S: Fn(&T, &Img, Point<f64>, Duration) -> Rgba<u8> + Clone + Send + Sync,
{
    fn process(&mut self, params: &mut RenderableParams, time: Duration) {
        (self.process)(&mut self.data, params, time);
    }
    fn get_pixel(&self, current_frame: &Img, uv_coords: Point<f64>, time: Duration) -> Rgba<u8> {
        (self.shader)(&self.data, current_frame, uv_coords, time)
    }
}
