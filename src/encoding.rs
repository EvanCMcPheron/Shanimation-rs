use core::slice::SlicePattern;

use image::{Rgb32FImage, RgbaImage};
use openh264::{
    encoder::{EncodedBitStream, Encoder, EncoderConfig, RateControlMode},
    formats::YUVBuffer,
};

pub(crate) fn rgba_to_yuv(image: RgbaImage) -> YUVBuffer {
    let dimensions = image.dimensions();
    YUVBuffer::with_rgb(
        dimensions.0 as usize,
        dimensions.1 as usize,
        image.into_raw().as_slice(),
    )
}


