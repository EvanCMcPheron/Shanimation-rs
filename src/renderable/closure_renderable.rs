use super::{
    Renderable,
    RenderableParams,
    super::Point,
    Img, Behaviour,
};
use std::time::Duration;
use image::Rgba;

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
