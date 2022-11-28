use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error("insufficient capacity (required: {required:?}, remaining: {remaining:?})")]
    InsufficientCapacity { required: usize, remaining: usize },
}

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("buffer too small (min: {min:?}, remaining: {remaining:?})")]
    BufferTooSmall { min: usize, remaining: usize },
    #[error("bad message code {code:?}")]
    BadMessageCode { code: u8 },
}
