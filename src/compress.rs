use std::os::raw::c_int;

use ps_buffer::Buffer;

use crate::error::CompressionError;
use crate::zstd;

// Fixed compression policy for deterministic output.
const ZSTD_LEVEL: c_int = 2;

/// Compresses `data` into an existing output buffer.
///
/// Returns the number of bytes written into `out_data`.
///
/// # Errors
///
/// Returns:
/// - [`CompressionError::InsufficientSpace`] if `out_data` is too small.
/// - [`CompressionError::CodecError`] if zstd fails for another reason.
pub fn compress_into(data: &[u8], out_data: &mut [u8]) -> Result<usize, CompressionError> {
    let size = match zstd::compress(data, out_data, ZSTD_LEVEL) {
        Ok(size) => size,
        Err(code) => {
            return if zstd::is_dst_too_small(code) {
                Err(CompressionError::InsufficientSpace)
            } else {
                Err(CompressionError::CodecError)
            };
        }
    };

    Ok(size)
}

/// Compresses `data` and allocates an output buffer for the compressed bytes.
///
/// # Errors
///
/// Returns:
/// - [`CompressionError::BufferError`] if output allocation fails.
/// - [`CompressionError::CodecError`] or [`CompressionError::InsufficientSpace`]
///   if zstd compression fails.
pub fn compress(data: &[u8]) -> Result<Buffer, CompressionError> {
    let out_size = zstd::compress_bound(data.len());
    let mut out_data = Buffer::alloc_uninit(out_size)?;

    let size = compress_into(data, &mut out_data)?;

    if size < out_size {
        out_data.truncate(size);
    }

    Ok(out_data)
}
