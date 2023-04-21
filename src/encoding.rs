use core::slice::SlicePattern;

use image::{Rgb, RgbImage, Rgba, RgbaImage};
pub use openh264::encoder::RateControlMode;
use openh264::{
    encoder::{EncodedBitStream, Encoder, EncoderConfig},
    formats::YUVBuffer,
};

pub(crate) fn rgba_to_rgb(image: RgbaImage) -> RgbImage {
    let mut ret = RgbImage::new(image.width(), image.height());
    image
        .enumerate_pixels()
        .for_each(|(x, y, p)| ret.put_pixel(x, y, Rgb([p.0[0], p.0[1], p.0[2]]))); // TODO: Might have to include some sort of alpha here
    ret
}

pub(crate) fn rgba_to_yuv(image: RgbaImage) -> YUVBuffer {
    let image = rgba_to_rgb(image);
    let dimensions = image.dimensions();
    YUVBuffer::with_rgb(
        dimensions.0 as usize,
        dimensions.1 as usize,
        image.into_raw().as_slice(),
    )
}
