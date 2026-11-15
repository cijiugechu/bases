# bases

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MSRV 1.88][msrv-badge]](https://www.whatrustisit.com)
[![Apache 2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/bases.svg
[crates-url]: https://crates.io/crates/bases
[docs-badge]: https://img.shields.io/docsrs/bases
[docs-url]: https://docs.rs/bases
[msrv-badge]: https://img.shields.io/badge/MSRV-1.88-green?logo=rust
[license-badge]: https://img.shields.io/badge/license-Apache--2.0-blue
[license-url]: https://www.apache.org/licenses/LICENSE-2.0
[actions-badge]: https://github.com/fast/bases/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/fast/bases/actions/workflows/ci.yml

Fast, strict RFC 4648 Base16, Base32, and Base64 codecs for Rust.

Bases provides configurable zero-sized family types (`base16::Base16`, `base32::Base32`, `base64::Base64`) parameterized by `policy` markers alongside ten crate-root profile aliases (`Base16Lower`, `Base16Upper`, `Base32`, `Base32Unpadded`, `Base32Hex`, `Base32HexUnpadded`, `Base64`, `Base64Unpadded`, `Base64Url`, `Base64UrlUnpadded`). Every profile exposes direct inherent methods (including `const fn` sizing and constructors) without requiring trait imports, and implements the sealed `Codec` trait for generic callers.

```rust
use bases::{Base64, Base64UrlUnpadded};

let mut buffer = [0_u8; 16];
let encoded = Base64::encode_into(b"hello", &mut buffer).unwrap();
assert_eq!(encoded, "aGVsbG8=");

let encoded = Base64UrlUnpadded::encode_into(b"hello", &mut buffer).unwrap();
assert_eq!(encoded, "aGVsbG8");
```

## API layers

- `encoded_len`, `max_decoded_len`, `encode_into`, `decode_into`, `decode_in_place`, and `decode_array` provide allocation-free operations, with `const fn` support on concrete profile `name`, `encoded_len`, `max_decoded_len`, `display`, `encoder`, and `decoder`.
- `encode`, `encode_append`, `decode`, and `decode_append` provide `String` and `Vec<u8>` helpers when the `alloc` feature is enabled.
- `Encoder<C>` and `Decoder<C>` process arbitrarily chunked input while retaining at most one partial quantum; the `std` feature adds `bases::io::{EncoderWriter, DecoderReader}` for `std::io::{Write, Read}`.
- `display` (`EncodedDisplay`) and `HexDisplay` (`LowerHex` `{:x}` and `UpperHex` `{:X}` only) write encoded data directly through `core::fmt`.
- `serialize`, `deserialize_array`, and (with `alloc`) `deserialize` integrate with `#[serde(with = "...")]` when the `serde` feature is enabled.

```rust
use bases::Base64;

let mut encoder = Base64::encoder();
let mut output = [0_u8; 4];
assert_eq!(encoder.update(b"fo", &mut output).unwrap(), "");
assert_eq!(encoder.update(b"o", &mut output).unwrap(), "Zm9v");
assert_eq!(encoder.finish(&mut output).unwrap(), "");
```

Bases is `#![no_std]` by default (`default = []`), exposing caller-buffer, in-place, fixed-array, streaming, and formatting APIs with zero default features. The `alloc` feature adds `String` and `Vec<u8>` helpers; `std` enables `alloc`, `bases::io::{EncoderWriter, DecoderReader}`, and runtime x86/x86_64 CPU feature detection; `serde` adds `serde_core` serialization and fixed-array deserialization in `no_std`, plus `Vec<u8>` deserialization when combined with `alloc`.

Successful `finish()` consumes the stream. On failure, `FinishError::into_inner()` recovers the unchanged encoder or decoder for retrying with a larger output buffer. Callers that do not need recovery can use `?` to convert `FinishError` into the underlying `EncodeError`, `DecodeError`, or `DecodeSliceError`, discarding the stream.

## Examples

Runnable examples live in the `examples` workspace crate. Each program uses local sample data and can run without arguments from the repository root:

```shell
cargo run --package examples --example url_cursor
```

| Example                                                | Application                                                                                 | APIs to start with                                       |
|--------------------------------------------------------|---------------------------------------------------------------------------------------------|----------------------------------------------------------|
| [`base64_field`](examples/src/base64_field.rs)         | Carry binary data in a JSON string field and reject a malformed field.                      | `Base64::encode`, `Base64::decode`, `DecodeError`        |
| [`url_cursor`](examples/src/url_cursor.rs)             | Encode a record ID into an unpadded URL-safe pagination cursor and recover it.              | `Base64UrlUnpadded::encode`, `decode_array`              |
| [`reusable_buffers`](examples/src/reusable_buffers.rs) | Reuse output buffers across a batch of records with different encoding profiles.            | `C: Codec`, sizing helpers, `encode_into`, `decode_into` |
| [`stream_io`](examples/src/stream_io.rs)               | Convert a file or network stream with bounded buffers and explicit finalization.            | `Encoder<C>`, `Decoder<C>`, `update`, `finish`           |
| [`format_bytes`](examples/src/format_bytes.rs)         | Format binary IDs and payloads in diagnostic output without an intermediate encoded string. | `HexDisplay` (`{:x}`/`{:X}`), `Base64::display`          |
| [`profiles`](examples/src/profiles.rs)                 | Compare Base16, Base32, Base64, and Base64URL representations.                              | `Base16Lower`, `Base16Upper`, `Base32`, `Base64`         |

The `stream_io` example also accepts `encode` or `decode` to transform standard input into standard output using padded Base64:

```shell
printf 'hello, world' | cargo run --quiet --package examples --example stream_io -- encode
printf 'aGVsbG8sIHdvcmxk' | cargo run --quiet --package examples --example stream_io -- decode
```

Its `encode` and `decode` helpers accept any `Read` and `Write`, including files and sockets. Successful updates consume the complete input chunk; `finish()` handles the final fragment at EOF. A decoding error can arrive after earlier output has been written, so callers should treat the destination as complete only after the whole operation succeeds.

## Strictness

Decoders reject whitespace, bytes outside the selected alphabet, misplaced or forbidden padding, and non-zero unused trailing bits. Padded and unpadded protocols use different profile types; `Base16Lower` and `Base16Upper` default to RFC 4648 case-insensitive decoding (`policy::Mixed`), while strict single-case Base16 and case-insensitive Base32 (such as RFC 5155 `Base32<ExtendedHex, Unpadded, Lower, Mixed>`) can be selected explicitly via `policy` markers.

Base encoding is not encryption. Bases does not provide constant-time guarantees and uses data-dependent branches, table lookups, and private scalar or SIMD backends (`aarch64` NEON, `x86`/`x86_64` SSSE3 and AVX2, and `wasm32` `simd128`).

## Architecture

[Architecture](ARCHITECTURE.md) records the public layers, private backend boundary, and performance contracts. See [CHANGELOG.md](CHANGELOG.md) for release notes.

## Benchmarks

Run `cargo x bench` to compare caller buffers, owned results, formatting, streaming, and invalid-input handling against other Rust codecs. The [benchmark guide](benchmarks/README.md) lists the workloads and explains how to read the timings.

## Minimum Rust version policy

The minimum supported Rust version is 1.88.0. It may be increased in a minor release, but not in a patch release.

## License

This project is licensed under [Apache License, Version 2.0](LICENSE).
