//! Deterministic Zstandard compression helpers for small payloads.
//!
//! This crate fixes compression policy to produce stable output for identical inputs.

mod compress;
mod decompress;
mod error;
mod zstd;

pub use compress::{compress, compress_into};
pub use decompress::{decompress, decompress_bounded, decompress_into};
pub use error::{CompressionError, DecompressionError};

#[cfg(test)]
mod tests;
