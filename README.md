# ps-compress

Deterministic Zstandard compression for small binary payloads.

This crate exposes a small API around zstd with a fixed compression policy
(zstd level 2), suitable for use-cases where the same input must always
compress to the same output bytes.

## API

- `compress(data: &[u8]) -> Result<Buffer, CompressionError>`
- `compress_into(data: &[u8], out_data: &mut [u8]) -> Result<usize, CompressionError>`
- `decompress(data: &[u8]) -> Result<Buffer, DecompressionError>`
- `decompress_bounded(data: &[u8], max_output_bytes: usize) -> Result<Buffer, DecompressionError>`
- `decompress_into(data: &[u8], out_data: &mut [u8]) -> Result<usize, DecompressionError>`

## Example

```rust
use ps_compress::{compress, decompress};

let input = b"example payload";
let compressed = compress(input)?;
let decompressed = decompress(&compressed)?;

assert_eq!(decompressed.as_slice(), input);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Determinism Notes

- Compression parameters are fixed by this crate.
- For strict cross-environment determinism, keep the crate dependency pinned so
  all environments use the same zstd implementation version.
