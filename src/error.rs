use ps_buffer::BufferError;
use thiserror::Error;

/// Errors returned by [`compress`](crate::compress) and [`compress_into`](crate::compress_into).
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CompressionError {
    /// Output buffer allocation failed.
    #[error(transparent)]
    BufferError(#[from] BufferError),
    /// The output buffer is too small for the compressed data.
    #[error("Insufficient buffer size, output too large")]
    InsufficientSpace,
    /// zstd reported a compression failure.
    #[error("Compression error")]
    CodecError,
}

/// Errors returned by [`decompress`](crate::decompress), [`decompress_bounded`](crate::decompress_bounded), and [`decompress_into`](crate::decompress_into).
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DecompressionError {
    /// Output buffer allocation failed.
    #[error(transparent)]
    BufferError(#[from] BufferError),
    /// The input is not valid zstd data or lacks required frame metadata.
    #[error("Decompression error: invalid data")]
    BadData,
    /// The decompressed size exceeds the caller-supplied maximum.
    #[error("Decompressed size {size} exceeds maximum {max}")]
    TooLarge { size: usize, max: usize },
    /// The output buffer is too small for the decompressed data.
    #[error("Insufficient buffer size, output too large")]
    InsufficientSpace,
}
