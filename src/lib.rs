pub use error_stack::{Context, IntoReport, Report, Result, ResultExt};
pub use error_stack_derive::ErrorStack;
pub use imageproc::point::Point;
//ffmpeg -reinit_filter 0 -f concat -safe 0 -i "ffmpeg.Txt" -vf "scale=1280:720:force_original_aspect_ratio=decrease:eval=frame,pad=1280:720:-1:-1:color=black:eval=frame,settb=AVTB,format=yuv420p" -r 15 output.mp4

pub mod frame_dictionary;
pub mod renderable;
pub mod scene;
