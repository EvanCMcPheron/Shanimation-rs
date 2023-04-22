use super::RenderableParams;
use super::{Behaviour, Point, Rgba};
use crate::scene::Img;
use error_stack::{IntoReport, Report, Result, ResultExt};
use error_stack_derive::ErrorStack;
use image::io::Reader;
use image::RgbaImage;
use std::path::Path;

#[derive(Debug, ErrorStack)]
#[error_message("There was an error with RenderableImage")]
pub enum RenderableImageError {
    ImageLoadingError,
}

pub struct RendreableImage {
    image: RgbaImage,
    process: Box<dyn Fn(&mut RgbaImage, &mut RenderableParams, std::time::Duration) + Send + Sync>,
}

impl RendreableImage {
    pub fn new<P: AsRef<Path> + ?Sized>(
        path: &P,
        process: Box<dyn Fn(&mut RgbaImage, &mut RenderableParams, std::time::Duration) + Send + Sync>,
    ) -> Result<Self, RenderableImageError> {
        let image = Reader::open(path)
            .into_report()
            .change_context(RenderableImageError::ImageLoadingError)
            .attach_printable_lazy(|| "Failed to open image")?
            .decode()
            .into_report()
            .change_context(RenderableImageError::ImageLoadingError)
            .attach_printable_lazy(|| "Failed to decode image")?
            .as_mut_rgba8()
            .ok_or(Report::new(RenderableImageError::ImageLoadingError))
            .attach_printable_lazy(|| "Failed to transcode image pixel format into RGB8")?
            .to_owned();
        Ok(Self {
            image,
            process: process,
        })
    }
}

impl Behaviour for RendreableImage {
    fn process(&mut self, renderable: &mut super::RenderableParams, time: std::time::Duration) {
        (self.process)(&mut self.image, renderable, time);
    }
    fn get_pixel(
        &self,
        _current_frame: &Img,
        uv_coords: Point<f64>,
        _time: std::time::Duration,
    ) -> Rgba<u8> {
        let image_coord: Point<u32> = uv_coords
            .map_y(|y| 1.0 - y) // Flip y
            .map_x(|x| x * self.image.width() as f64)
            .map_y(|y| y * self.image.height() as f64)
            .map_both(|v| v as u32);

        *self.image.get_pixel(image_coord.x, image_coord.y)
    }
}
