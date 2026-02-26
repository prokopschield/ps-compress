#![allow(clippy::expect_used)]

use crate::{
    compress, compress_into, decompress, decompress_bounded, decompress_into, CompressionError,
    DecompressionError,
};

fn lcg_bytes(len: usize) -> Vec<u8> {
    let mut x: u64 = 0x1234_5678_9ABC_DEF0;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        x = x.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        out.push(x.to_le_bytes()[4]);
    }
    out
}

fn sample_payloads() -> Vec<Vec<u8>> {
    let mut out = vec![
        Vec::new(),
        vec![0u8; 1],
        vec![0u8; 64],
        vec![0u8; 4096],
        b"short".to_vec(),
        b"deterministic payload deterministic payload".to_vec(),
        (u8::MIN..=u8::MAX).collect(),
        (0u16..4096)
            .map(|i| u8::try_from(i % 251).expect("value is modulo 251"))
            .collect(),
        lcg_bytes(31),
        lcg_bytes(512),
        lcg_bytes(4096),
    ];

    let mut mixed = vec![0u8; 4096];
    mixed[2048..].copy_from_slice(&lcg_bytes(2048));
    out.push(mixed);

    let mut texty = Vec::new();
    let base = b"{\"id\":123,\"name\":\"example\",\"active\":true,\"roles\":[\"a\",\"b\"]}\n";
    while texty.len() < 4096 {
        let rem = 4096 - texty.len();
        if rem >= base.len() {
            texty.extend_from_slice(base);
        } else {
            texty.extend_from_slice(&base[..rem]);
        }
    }
    out.push(texty);

    out
}

fn assert_round_trip(payload: &[u8]) {
    let compressed = compress(payload).expect("compression should succeed");

    let decompressed = decompress(&compressed).expect("decompression should succeed");
    assert_eq!(decompressed.as_slice(), payload);

    let mut out_exact = vec![0u8; payload.len()];
    let written_exact =
        decompress_into(&compressed, &mut out_exact).expect("decompress_into should succeed");
    assert_eq!(written_exact, payload.len());
    assert_eq!(out_exact, payload);

    let mut out_larger = vec![0u8; payload.len() + 17];
    let written_larger =
        decompress_into(&compressed, &mut out_larger).expect("decompress_into should succeed");
    assert_eq!(written_larger, payload.len());
    assert_eq!(&out_larger[..written_larger], payload);
}

#[test]
fn round_trip_many_payloads() {
    for payload in sample_payloads() {
        assert_round_trip(&payload);
    }
}

#[test]
fn compression_is_deterministic_across_many_payloads() {
    for payload in sample_payloads() {
        let baseline = compress(&payload).expect("compression should succeed");
        for _ in 0..32 {
            let next = compress(&payload).expect("compression should succeed");
            assert_eq!(next.as_slice(), baseline.as_slice());
        }
    }
}

#[test]
fn compress_into_matches_compress_output() {
    for payload in sample_payloads() {
        let compressed = compress(&payload).expect("compression should succeed");
        let mut out = vec![0u8; payload.len().saturating_add(1024)];
        let written = compress_into(&payload, &mut out).expect("compress_into should succeed");
        assert_eq!(written, compressed.len());
        assert_eq!(&out[..written], compressed.as_slice());
    }
}

#[test]
fn compress_into_reports_insufficient_space() {
    let input = lcg_bytes(1024);
    let compressed = compress(&input).expect("compression should succeed");
    assert!(!compressed.is_empty());

    let mut too_small = vec![0u8; compressed.len() - 1];
    let err = compress_into(&input, &mut too_small).expect_err("must fail");
    assert_eq!(err, CompressionError::InsufficientSpace);
}

#[test]
fn decompress_into_exact_size_succeeds() {
    let input = lcg_bytes(2048);
    let compressed = compress(&input).expect("compression should succeed");

    let mut out = vec![0u8; input.len()];
    let written = decompress_into(&compressed, &mut out).expect("decompress_into should succeed");

    assert_eq!(written, input.len());
    assert_eq!(out, input);
}

#[test]
fn decompress_into_larger_buffer_returns_exact_written_size() {
    let input = b"small payload small payload small payload".to_vec();
    let compressed = compress(&input).expect("compression should succeed");

    let mut out = vec![0xAAu8; input.len() + 32];
    let written = decompress_into(&compressed, &mut out).expect("decompress_into should succeed");

    assert_eq!(written, input.len());
    assert_eq!(&out[..written], input.as_slice());
}

#[test]
fn decompress_into_reports_insufficient_space() {
    let input = b"this payload is a little longer than output";
    let compressed = compress(input).expect("compression should succeed");

    let mut too_small = [0u8; 8];
    let err = decompress_into(&compressed, &mut too_small).expect_err("must fail");

    assert_eq!(err, DecompressionError::InsufficientSpace);
}

#[test]
fn decompress_rejects_empty_input() {
    let err = decompress(&[]).expect_err("must fail");
    assert_eq!(err, DecompressionError::BadData);
}

#[test]
fn decompress_rejects_non_zstd_data() {
    let garbage = lcg_bytes(128);
    let err = decompress(&garbage).expect_err("must fail");
    assert_eq!(err, DecompressionError::BadData);
}

#[test]
fn decompress_rejects_truncated_frame() {
    let input = lcg_bytes(2048);
    let compressed = compress(&input).expect("compression should succeed");
    let truncated = &compressed[..compressed.len() - 1];

    let err = decompress(truncated).expect_err("must fail");
    assert_eq!(err, DecompressionError::BadData);
}

#[test]
fn decompress_rejects_corrupted_magic() {
    let input = lcg_bytes(512);
    let compressed = compress(&input).expect("compression should succeed");
    let mut corrupted = compressed.to_vec();
    corrupted[0] ^= 0xFF;

    let err = decompress(&corrupted).expect_err("must fail");
    assert_eq!(err, DecompressionError::BadData);
}

#[test]
fn decompress_into_rejects_corrupted_magic_with_bad_data() {
    let input = lcg_bytes(512);
    let compressed = compress(&input).expect("compression should succeed");
    let mut corrupted = compressed.to_vec();
    corrupted[0] ^= 0xAA;

    let mut out = vec![0u8; input.len()];
    let err = decompress_into(&corrupted, &mut out).expect_err("must fail");
    assert_eq!(err, DecompressionError::BadData);
}

#[test]
fn zero_length_payload_round_trip() {
    let input: [u8; 0] = [];
    let compressed = compress(&input).expect("compression should succeed");
    let decompressed = decompress(&compressed).expect("decompression should succeed");
    assert_eq!(decompressed.len(), 0);
}

#[test]
fn decompress_into_works_for_zero_length_payload() {
    let input: [u8; 0] = [];
    let compressed = compress(&input).expect("compression should succeed");
    let mut out = [0u8; 0];
    let written = decompress_into(&compressed, &mut out).expect("decompression should succeed");
    assert_eq!(written, 0);
}

#[test]
fn decompression_not_failing_for_many_sizes() {
    let mut sizes = vec![0usize, 1, 2, 3, 4, 7, 8, 15, 16, 31, 32, 63, 64];
    sizes.extend([
        127, 128, 255, 256, 511, 512, 1023, 1024, 2047, 2048, 3072, 4095, 4096,
    ]);

    for size in sizes {
        assert_round_trip(&lcg_bytes(size));
    }
}

#[test]
fn decompression_not_failing_for_many_repeated_calls() {
    let payload = lcg_bytes(4096);
    let compressed = compress(&payload).expect("compression should succeed");

    for _ in 0..128 {
        let decompressed = decompress(&compressed).expect("decompression should succeed");
        assert_eq!(decompressed.as_slice(), payload.as_slice());

        let mut out = vec![0u8; payload.len()];
        let written = decompress_into(&compressed, &mut out).expect("decompress_into should work");
        assert_eq!(written, payload.len());
        assert_eq!(out, payload);
    }
}

#[test]
fn decompression_fails_for_many_garbled_inputs() {
    for size in [
        1usize, 2, 3, 4, 7, 8, 15, 16, 32, 64, 127, 128, 256, 512, 1024,
    ] {
        let garbled = lcg_bytes(size);
        let err = decompress(&garbled).expect_err("must fail");
        assert_eq!(err, DecompressionError::BadData);
    }
}

#[test]
fn decompression_fails_for_all_truncated_prefixes() {
    let payload = lcg_bytes(1024);
    let compressed = compress(&payload).expect("compression should succeed");

    for prefix_len in 0..compressed.len() {
        let err = decompress(&compressed[..prefix_len]).expect_err("must fail");
        assert_eq!(err, DecompressionError::BadData);
    }
}

#[test]
fn decompression_into_fails_for_many_garbled_inputs() {
    for size in [1usize, 5, 9, 17, 31, 65, 129, 257] {
        let garbled = lcg_bytes(size);
        let mut out = vec![0u8; 4096];
        let err = decompress_into(&garbled, &mut out).expect_err("must fail");
        assert_eq!(err, DecompressionError::BadData);
    }
}

#[test]
fn compression_does_not_fail_for_garbled_inputs() {
    for size in [0usize, 1, 2, 3, 7, 8, 15, 16, 64, 256, 1024, 4096] {
        let garbled = lcg_bytes(size);
        let compressed = compress(&garbled).expect("compression should succeed");
        let round_trip = decompress(&compressed).expect("decompression should succeed");
        assert_eq!(round_trip.as_slice(), garbled.as_slice());
    }
}

#[test]
fn decompress_bounded_succeeds_when_within_limit() {
    let payload = lcg_bytes(1024);
    let compressed = compress(&payload).expect("compression should succeed");

    let out = decompress_bounded(&compressed, 2048).expect("decompression should succeed");
    assert_eq!(out.as_slice(), payload.as_slice());
}

#[test]
fn decompress_bounded_succeeds_when_limit_is_exact() {
    let payload = lcg_bytes(777);
    let compressed = compress(&payload).expect("compression should succeed");

    let out = decompress_bounded(&compressed, payload.len()).expect("decompression should succeed");
    assert_eq!(out.as_slice(), payload.as_slice());
}

#[test]
fn decompress_bounded_rejects_when_limit_too_small() {
    let payload = lcg_bytes(1024);
    let compressed = compress(&payload).expect("compression should succeed");

    let err = decompress_bounded(&compressed, 1023).expect_err("must fail");
    assert_eq!(
        err,
        DecompressionError::TooLarge {
            size: 1024,
            max: 1023
        }
    );
}

#[test]
fn decompress_bounded_rejects_bad_data() {
    let garbled = lcg_bytes(128);
    let err = decompress_bounded(&garbled, 4096).expect_err("must fail");
    assert_eq!(err, DecompressionError::BadData);
}
