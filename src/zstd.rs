use zstd_safe::zstd_sys::{ZSTD_ErrorCode, ZSTD_getErrorCode};

pub fn compress_bound(input_size: usize) -> usize {
    zstd_safe::compress_bound(input_size)
}

pub fn compress(
    data: &[u8],
    out_data: &mut [u8],
    level: zstd_safe::CompressionLevel,
) -> zstd_safe::SafeResult {
    zstd_safe::compress(out_data, data, level)
}

pub fn decompress(data: &[u8], out_data: &mut [u8]) -> zstd_safe::SafeResult {
    zstd_safe::decompress(out_data, data)
}

pub fn frame_content_size(data: &[u8]) -> Option<usize> {
    match zstd_safe::get_frame_content_size(data) {
        Ok(Some(size)) => usize::try_from(size).ok(),
        Ok(None) | Err(_) => None,
    }
}

pub fn is_dst_too_small(code: usize) -> bool {
    // SAFETY: zstd accepts any result code produced by zstd APIs.
    unsafe { ZSTD_getErrorCode(code) == ZSTD_ErrorCode::ZSTD_error_dstSize_tooSmall }
}
