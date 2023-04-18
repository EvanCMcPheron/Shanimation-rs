use error_stack::{Context, IntoReport, Report, Result, ResultExt};
use error_stack_derive::ErrorStack;
use shanimation::frame_dictionary::FrameDict;

#[derive(Debug, ErrorStack)]
#[error_message("Error occured in main fn")]
pub enum MainError {
    FrameDictCreation,
}

fn main() -> Result<(), MainError> {
    FrameDict { frame_count: 20 }
        .save()
        .change_context(MainError::FrameDictCreation)?;

    

    Ok(())
}
