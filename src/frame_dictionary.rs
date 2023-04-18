use error_stack::{IntoReport, Result, ResultExt};
use error_stack_derive::ErrorStack;
pub use imageproc::point::Point;
use std::fs::{DirBuilder, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug, ErrorStack)]
#[error_message("An error occured when compiling the frame dictionary")]
pub enum FrameDictError {
    FrameDirectoryCreation,
    FrameDictionaryCreation,
}

pub struct FrameDict {
    pub frame_count: usize,
}

impl FrameDict {
    pub fn save(&self) -> Result<(), FrameDictError> {
        if !Path::new("./frames").exists() {
            DirBuilder::new()
                .recursive(true)
                .create("./frames")
                .into_report()
                .change_context(FrameDictError::FrameDirectoryCreation)
                .attach_printable_lazy(|| "Failed to create './frames directory")?;
        }

        let mut file = File::create("./frames/FrameDict.txt")
            .into_report()
            .change_context(FrameDictError::FrameDictionaryCreation)
            .attach_printable_lazy(|| "Failed to create file")?;

        let txt: String = (1..=self.frame_count)
            .map(|i| format!("file '{i}.png'\n"))
            .collect();

        file.write_all(txt.as_bytes())
            .into_report()
            .change_context(FrameDictError::FrameDictionaryCreation)
            .attach_printable_lazy(|| "Failed to write to file")?;

        Ok(())
    }
}
