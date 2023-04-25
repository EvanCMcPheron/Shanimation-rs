use crate::prelude::*;
use std::time::Duration;

/// Ex:
/// ```
/// use shanimation_rs::prelude::*;
/// 
/// ClosureRenderable {
///    data: (),
///    process: |data, params, time, scene, abs_position| {
///        params.rotation = time.as_secs_f64() * 2.0;
///    },
///    shader: |data, frame, uv, time, abs_position| -> Rgba<u8> {
///        let p = uv.map_both(|v| (v * 255.0) as u8);
///        Rgba([255, p.x, p.y, 255])
///    },
/// };
///```
#[derive(Clone)]
pub struct ClosureRenderable<T, P, S>
where  
    T: Clone + Send + Sync,
    P: Fn(&mut T, &mut RenderableParams, Duration, &Scene, Point<isize>) + Clone + Send + Sync,
    S: Fn(&T, &Img, Point<f64>, Duration, Point<isize>) -> Rgba<u8> + Clone + Send + Sync,
{
    pub data: T,
    pub process: P,
    pub shader: S,
}

impl<T, P, S> Behaviour for ClosureRenderable<T, P, S>
where  
    T: Clone + Send + Sync,
    P: Fn(&mut T, &mut RenderableParams, Duration, &Scene, Point<isize>) + Clone + Send + Sync,
    S: Fn(&T, &Img, Point<f64>, Duration, Point<isize>) -> Rgba<u8> + Clone + Send + Sync,
{
    fn process(&mut self, params: &mut RenderableParams, time: Duration, scene: &Scene, abs_position: Point<isize>) {
        (self.process)(&mut self.data, params, time, scene, abs_position);
    }
    fn get_pixel(&self, current_frame: &Img, uv_coords: Point<f64>, time: Duration, abs_position: Point<isize>) -> Rgba<u8> {
        (self.shader)(&self.data, current_frame, uv_coords, time, abs_position)
    }
}
