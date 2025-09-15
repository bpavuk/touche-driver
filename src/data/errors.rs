use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToucheError {
    #[error("driver loop interrupted")]
    LoopInterrupted,
    #[error("invalid size data")]
    InvalidSizeData
}

