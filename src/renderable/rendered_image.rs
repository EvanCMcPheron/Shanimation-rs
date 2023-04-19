use super::{Point, Renderable, Rgba};
use error_stack::{Context, IntoReport, Report, Result, ResultExt};
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
}

impl RendreableImage {
    pub fn new<P: AsRef<Path>>(path: &P) -> Result<Self, RenderableImageError> {
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
        Ok(Self { image })
    }
}
