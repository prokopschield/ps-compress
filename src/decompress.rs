use ps_buffer::Buffer;

use crate::error::DecompressionError;
use crate::zstd;

fn required_out_size(data: &[u8]) -> Result<usize, DecompressionError> {
    zstd::frame_content_size(data).ok_or(DecompressionError::BadData)
}

fn required_out_size_bounded(
    data: &[u8],
    max_output_bytes: usize,
) -> Result<usize, DecompressionError> {
    let out_size = required_out_size(data)?;
    if out_size > max_output_bytes {
        return Err(DecompressionError::TooLarge {
            size: out_size,
            max: max_output_bytes,
        });
    }
    Ok(out_size)
}

/// Decompresses `data` into an existing output buffer.
///
/// Returns the number of bytes written into `out_data`.
///
/// # Errors
///
/// Returns:
/// - [`DecompressionError::BadData`] if input is invalid or missing required frame metadata.
/// - [`DecompressionError::InsufficientSpace`] if `out_data` is too small.
pub fn decompress_into(data: &[u8], out_data: &mut [u8]) -> Result<usize, DecompressionError> {
    let expected_size = required_out_size(data)?;
    if expected_size > out_data.len() {
        return Err(DecompressionError::InsufficientSpace);
    }

    let size = match zstd::decompress(data, out_data) {
        Ok(size) => size,
        Err(code) => {
            return if zstd::is_dst_too_small(code) {
                Err(DecompressionError::InsufficientSpace)
            } else {
                Err(DecompressionError::BadData)
            };
        }
    };

    Ok(size)
}

/// Decompresses `data`, allocating an output buffer sized from frame metadata.
///
/// # Errors
///
/// Returns:
/// - [`DecompressionError::BadData`] if input is invalid or frame size metadata is unavailable.
/// - [`DecompressionError::BufferError`] if output allocation fails.
/// - [`DecompressionError::InsufficientSpace`] if decompression reports insufficient output space.
pub fn decompress(data: &[u8]) -> Result<Buffer, DecompressionError> {
    let out_size = required_out_size(data)?;
    let mut out_data = Buffer::alloc_uninit(out_size)?;

    let size = decompress_into(data, &mut out_data)?;

    if size < out_size {
        out_data.truncate(size);
    }

    Ok(out_data)
}

/// Decompresses `data`, allocating an output buffer up to `max_output_bytes`.
///
/// # Errors
///
/// Returns:
/// - [`DecompressionError::BadData`] if input is invalid or frame size metadata is unavailable.
/// - [`DecompressionError::TooLarge`] if frame size exceeds `max_output_bytes`.
/// - [`DecompressionError::BufferError`] if output allocation fails.
/// - [`DecompressionError::InsufficientSpace`] if decompression reports insufficient output space.
pub fn decompress_bounded(
    data: &[u8],
    max_output_bytes: usize,
) -> Result<Buffer, DecompressionError> {
    let out_size = required_out_size_bounded(data, max_output_bytes)?;
    let mut out_data = Buffer::alloc_uninit(out_size)?;

    let size = decompress_into(data, &mut out_data)?;

    if size < out_size {
        out_data.truncate(size);
    }

    Ok(out_data)
}
