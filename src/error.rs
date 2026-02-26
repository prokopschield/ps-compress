use ps_buffer::BufferError;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CompressionError {
    #[error(transparent)]
    BufferError(#[from] BufferError),
    #[error("Insufficient buffer size, output too large")]
    InsufficientSpace,
    #[error("Compression error")]
    CodecError,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DecompressionError {
    #[error(transparent)]
    BufferError(#[from] BufferError),
    #[error("Decompression error: invalid data")]
    BadData,
    #[error("Decompressed size {size} exceeds maximum {max}")]
    TooLarge { size: usize, max: usize },
    #[error("Insufficient buffer size, output too large")]
    InsufficientSpace,
}
