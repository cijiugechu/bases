# Codec benchmarks

Run the same fixed workload for every implementation in a scenario. The results give elapsed milliseconds for a batch of messages. Compare times within one row, since each row can select a different round count.

| Workload             | Raw message sizes     | Work per round                                                   | Peers                                                                                  |
|----------------------|-----------------------|------------------------------------------------------------------|----------------------------------------------------------------------------------------|
| Caller buffers       | 64 B, 1 KiB, 16 KiB   | Encode or decode into a reusable byte slice.                     | `data-encoding`, `hex`, `faster-hex`, `const-hex`, RustCrypto, `base64`, `base64-simd` |
| Owned output         | 8 B, 32 B, 1 KiB      | Return and drop a fresh string or byte vector for each message.  | `data-encoding`, `hex`, `const-hex`, `base16ct`, `base32ct`, `base64`                  |
| Display              | 32 B, 1 KiB           | Format lowercase hex or padded Base64 into a reused string.      | `data-encoding`, `base16ct` or `base64`                                                |
| Streaming Base64     | 16 KiB                | Process chunks of at most 257 bytes into a fixed destination.    | `base64` `EncoderWriter` and `DecoderReader`                                           |
| Invalid Base64 input | 1 KiB before encoding | Reject `!` at the first or last data symbol of a padded message. | `data-encoding`, `base64`                                                              |

Base16 and Base32 caller-buffer cases use 1 KiB. Base64 uses all three sizes. Owned workloads cover a 32-byte hex field, 1 KiB of padded Base32 or Base64, and an 8-byte unpadded URL-safe cursor. Owned decoders receive a string. Streaming includes finalization or reading to EOF, including the I/O adapter's buffering. The invalid-input cases check acceptance of the original message before corrupting one symbol.

```shell
cargo x bench --no-run
cargo x bench -- --list
cargo x bench -- --test
cargo x bench -- --repeat 5 --raw
cargo x bench -- base64/stream --repeat 5 --raw
```

Following [Benchmark In Milliseconds](https://matklad.github.io/2026/10/05/benchmark-milliseconds.html), choose fixed round counts that take roughly 300 ms on the reference implementation. To size a new case, use `--baseline-only --rounds N`. Keep the counts fixed across peers and revisions.

Each timed sample creates the same deterministic input and a known output, runs all rounds, checks the result and drops its buffers. The runner interleaves peers, rotates the starting order and reports the median and full range in integer milliseconds. `--raw` includes the first sample and every outlier. The elapsed-time ratio uses unrounded medians relative to `bases`, so `0.80x` means 20% less time for the same work. CI uses `--test` to verify the bytes or rejection once per case.

Base16 cases use lowercase output (`Base16Lower` and `HEXLOWER_PERMISSIVE`) because `hex::encode_to_slice` emits lowercase hex only. `faster-hex` is compared on caller-provided slice buffers (`hex_encode` and `hex_decode`) rather than `Vec`-returning owned decode because it exposes slice-buffer decoding without a first-party `Vec`-returning decode helper, whereas `hex` and `const-hex` appear in both caller-buffer and owned Base16 rows. The `base64` peer uses its standard general-purpose engine, and the separate `base64-simd` crate uses its default CPU dispatch. RustCrypto's `base16ct`, `base32ct` and `base64ct` are designed for best-effort constant-time operation. Valid-input rows use the same wire bytes for all peers. They measure neither the full malformed-input policies nor timing side channels.
