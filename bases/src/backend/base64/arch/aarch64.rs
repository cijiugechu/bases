// Copyright 2026 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! AArch64 NEON Base64 kernels.

use core::arch::aarch64::uint8x16_t;
use core::arch::aarch64::uint8x16x3_t;
use core::arch::aarch64::uint8x16x4_t;
use core::arch::aarch64::vaddq_u8;
use core::arch::aarch64::vandq_u8;
use core::arch::aarch64::vceqq_u8;
use core::arch::aarch64::vdupq_n_u8;
use core::arch::aarch64::vld1q_u8;
use core::arch::aarch64::vld3q_u8;
use core::arch::aarch64::vld4q_u8;
use core::arch::aarch64::vminq_u8;
use core::arch::aarch64::vminvq_u8;
use core::arch::aarch64::vorrq_u8;
use core::arch::aarch64::vqtbl1q_u8;
use core::arch::aarch64::vqtbl4q_u8;
use core::arch::aarch64::vshlq_n_u8;
use core::arch::aarch64::vshrq_n_u8;
use core::arch::aarch64::vst3q_u8;
use core::arch::aarch64::vst4q_u8;
use core::arch::aarch64::vsubq_u8;

const STANDARD: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URL_SAFE: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

const HI_MASK: [u8; 16] = [0, 0, 1, 2, 4, 8, 4, 16, 0, 0, 0, 0, 0, 0, 0, 0];
const LO_MASK_STD: [u8; 16] = [
    0x1a, 0x1e, 0x1e, 0x1e, 0x1e, 0x1e, 0x1e, 0x1e, 0x1e, 0x1e, 0x1c, 0x05, 0x04, 0x04, 0x04, 0x05,
];
const LO_MASK_URL: [u8; 16] = [
    0x1a, 0x1e, 0x1e, 0x1e, 0x1e, 0x1e, 0x1e, 0x1e, 0x1e, 0x1e, 0x1c, 0x04, 0x04, 0x05, 0x04, 0x0c,
];

const ROLL_STD: [u8; 16] = [0, 0, 19, 4, 0xbf, 0xbf, 0xb9, 0xb9, 0, 0, 0, 0, 0, 0, 0, 0];
const ROLL_URL: [u8; 16] = [0, 0, 17, 4, 0xbf, 0xbf, 0xb9, 0xb9, 0, 0, 0, 0, 0, 0, 0, 0];

pub fn encode<const URL_SAFE_ALPHABET: bool>(input: &[u8], output: &mut [u8]) -> (usize, usize) {
    let chunks = (input.len() / 48).min(output.len() / 64);
    if chunks == 0 {
        return (0, 0);
    }

    // SAFETY: `chunks * 48 <= input.len()` and `chunks * 64 <= output.len()`, and NEON is baseline.
    unsafe {
        encode_neon::<URL_SAFE_ALPHABET>(input.as_ptr(), output.as_mut_ptr(), chunks);
    }
    (chunks * 48, chunks * 64)
}

pub fn decode<const URL_SAFE_ALPHABET: bool>(input: &[u8], output: &mut [u8]) -> (usize, usize) {
    let chunks = (input.len() / 64).min(output.len() / 48);
    if chunks == 0 {
        return (0, 0);
    }

    // SAFETY: `chunks * 64 <= input.len()` and `chunks * 48 <= output.len()`, and NEON is baseline.
    let done =
        unsafe { decode_neon::<URL_SAFE_ALPHABET>(input.as_ptr(), output.as_mut_ptr(), chunks) };
    (done * 64, done * 48)
}

pub fn decode_in_place<const URL_SAFE_ALPHABET: bool>(
    buf: &mut [u8],
    complete_len: usize,
) -> (usize, usize) {
    let chunks = complete_len.min(buf.len()) / 64;
    if chunks == 0 {
        return (0, 0);
    }

    // SAFETY: `chunks * 64 <= buf.len()`. Every iteration loads 64 bytes before storing 48 bytes,
    // and `out_ptr` advances by 48 while `in_ptr` advances by 64.
    let done = unsafe { decode_in_place_neon::<URL_SAFE_ALPHABET>(buf.as_mut_ptr(), chunks) };
    (done * 64, done * 48)
}

/// # Safety
///
/// Caller must ensure `in_ptr` is valid for `chunks * 48` reads and `out_ptr` is valid for
/// `chunks * 64` non-overlapping writes, with NEON enabled.
#[target_feature(enable = "neon")]
unsafe fn encode_neon<const URL_SAFE_ALPHABET: bool>(
    mut in_ptr: *const u8,
    mut out_ptr: *mut u8,
    chunks: usize,
) {
    let alpha_ptr = if URL_SAFE_ALPHABET {
        &URL_SAFE
    } else {
        &STANDARD
    }
    .as_ptr();
    // SAFETY: Caller guarantees sufficient readable/writable bytes and `alpha_ptr` points to 64
    // bytes.
    unsafe {
        let alphabet_tbl = uint8x16x4_t(
            vld1q_u8(alpha_ptr),
            vld1q_u8(alpha_ptr.add(16)),
            vld1q_u8(alpha_ptr.add(32)),
            vld1q_u8(alpha_ptr.add(48)),
        );
        let mask_3f = vdupq_n_u8(0x3f);

        for _ in 0..chunks {
            let uint8x16x3_t(b0, b1, b2) = vld3q_u8(in_ptr);
            let idx0 = vshrq_n_u8::<2>(b0);
            let idx1 = vandq_u8(vorrq_u8(vshlq_n_u8::<4>(b0), vshrq_n_u8::<4>(b1)), mask_3f);
            let idx2 = vandq_u8(vorrq_u8(vshlq_n_u8::<2>(b1), vshrq_n_u8::<6>(b2)), mask_3f);
            let idx3 = vandq_u8(b2, mask_3f);

            let out0 = vqtbl4q_u8(alphabet_tbl, idx0);
            let out1 = vqtbl4q_u8(alphabet_tbl, idx1);
            let out2 = vqtbl4q_u8(alphabet_tbl, idx2);
            let out3 = vqtbl4q_u8(alphabet_tbl, idx3);
            vst4q_u8(out_ptr, uint8x16x4_t(out0, out1, out2, out3));

            in_ptr = in_ptr.add(48);
            out_ptr = out_ptr.add(64);
        }
    }
}

/// # Safety
///
/// Caller must ensure `in_ptr` is valid for `chunks * 64` reads and `out_ptr` is valid for
/// `chunks * 48` non-overlapping writes, with NEON enabled.
#[target_feature(enable = "neon")]
unsafe fn decode_neon<const URL_SAFE_ALPHABET: bool>(
    mut in_ptr: *const u8,
    mut out_ptr: *mut u8,
    chunks: usize,
) -> usize {
    let lo_mask_ptr = if URL_SAFE_ALPHABET {
        &LO_MASK_URL
    } else {
        &LO_MASK_STD
    }
    .as_ptr();
    let roll_ptr = if URL_SAFE_ALPHABET {
        &ROLL_URL
    } else {
        &ROLL_STD
    }
    .as_ptr();
    let mut done = 0;

    // SAFETY: Caller guarantees `chunks * 64` reads at `in_ptr`, `chunks * 48` writes at `out_ptr`,
    // and lookup tables are 16-byte static arrays.
    unsafe {
        let hi_mask_tbl = vld1q_u8(HI_MASK.as_ptr());
        let lo_mask_tbl = vld1q_u8(lo_mask_ptr);
        let roll_tbl = vld1q_u8(roll_ptr);
        let mask_0f = vdupq_n_u8(0x0f);
        let special_ch = vdupq_n_u8(if URL_SAFE_ALPHABET { b'_' } else { b'/' });
        let special_adj = vdupq_n_u8(if URL_SAFE_ALPHABET { 33 } else { 3 });

        while done < chunks {
            let uint8x16x4_t(c0, c1, c2, c3) = vld4q_u8(in_ptr);
            let Some(packed) = decode_chunk_64_neon::<URL_SAFE_ALPHABET>(
                c0,
                c1,
                c2,
                c3,
                hi_mask_tbl,
                lo_mask_tbl,
                roll_tbl,
                mask_0f,
                special_ch,
                special_adj,
            ) else {
                break;
            };
            vst3q_u8(out_ptr, packed);
            in_ptr = in_ptr.add(64);
            out_ptr = out_ptr.add(48);
            done += 1;
        }
    }
    done
}

/// # Safety
///
/// Caller must ensure `ptr` is valid for `chunks * 64` reads and writes, with NEON enabled.
#[target_feature(enable = "neon")]
unsafe fn decode_in_place_neon<const URL_SAFE_ALPHABET: bool>(
    ptr: *mut u8,
    chunks: usize,
) -> usize {
    let lo_mask_ptr = if URL_SAFE_ALPHABET {
        &LO_MASK_URL
    } else {
        &LO_MASK_STD
    }
    .as_ptr();
    let roll_ptr = if URL_SAFE_ALPHABET {
        &ROLL_URL
    } else {
        &ROLL_STD
    }
    .as_ptr();
    let mut in_ptr: *const u8 = ptr;
    let mut out_ptr: *mut u8 = ptr;
    let mut done = 0;

    // SAFETY: Caller guarantees `chunks * 64` readable/writable bytes from `ptr`. Each iteration
    // loads `in_ptr[0..64]` completely before writing `out_ptr[0..48]`, and `out_ptr` lags behind
    // `in_ptr` by at least `16 * done` bytes.
    unsafe {
        let hi_mask_tbl = vld1q_u8(HI_MASK.as_ptr());
        let lo_mask_tbl = vld1q_u8(lo_mask_ptr);
        let roll_tbl = vld1q_u8(roll_ptr);
        let mask_0f = vdupq_n_u8(0x0f);
        let special_ch = vdupq_n_u8(if URL_SAFE_ALPHABET { b'_' } else { b'/' });
        let special_adj = vdupq_n_u8(if URL_SAFE_ALPHABET { 33 } else { 3 });

        while done < chunks {
            let uint8x16x4_t(c0, c1, c2, c3) = vld4q_u8(in_ptr);
            let Some(packed) = decode_chunk_64_neon::<URL_SAFE_ALPHABET>(
                c0,
                c1,
                c2,
                c3,
                hi_mask_tbl,
                lo_mask_tbl,
                roll_tbl,
                mask_0f,
                special_ch,
                special_adj,
            ) else {
                break;
            };
            vst3q_u8(out_ptr, packed);
            in_ptr = in_ptr.add(64);
            out_ptr = out_ptr.add(48);
            done += 1;
        }
    }
    done
}

/// # Safety
///
/// Requires NEON target feature.
#[inline]
#[target_feature(enable = "neon")]
unsafe fn decode_lane_neon<const URL_SAFE_ALPHABET: bool>(
    c: uint8x16_t,
    hi_mask_tbl: uint8x16_t,
    lo_mask_tbl: uint8x16_t,
    roll_tbl: uint8x16_t,
    mask_0f: uint8x16_t,
    special_ch: uint8x16_t,
    special_adj: uint8x16_t,
) -> (uint8x16_t, uint8x16_t) {
    let hi = vshrq_n_u8::<4>(c);
    let lo = vandq_u8(c, mask_0f);
    let chk = vandq_u8(vqtbl1q_u8(hi_mask_tbl, hi), vqtbl1q_u8(lo_mask_tbl, lo));
    let base = vaddq_u8(c, vqtbl1q_u8(roll_tbl, hi));
    let adj = vandq_u8(vceqq_u8(c, special_ch), special_adj);
    let val = if URL_SAFE_ALPHABET {
        vaddq_u8(base, adj)
    } else {
        vsubq_u8(base, adj)
    };
    (chk, val)
}

/// # Safety
///
/// Requires NEON target feature.
#[inline]
#[target_feature(enable = "neon")]
#[allow(clippy::too_many_arguments)]
unsafe fn decode_chunk_64_neon<const URL_SAFE_ALPHABET: bool>(
    c0: uint8x16_t,
    c1: uint8x16_t,
    c2: uint8x16_t,
    c3: uint8x16_t,
    hi_mask_tbl: uint8x16_t,
    lo_mask_tbl: uint8x16_t,
    roll_tbl: uint8x16_t,
    mask_0f: uint8x16_t,
    special_ch: uint8x16_t,
    special_adj: uint8x16_t,
) -> Option<uint8x16x3_t> {
    // SAFETY: Caller guarantees NEON is enabled.
    unsafe {
        let (chk0, v0) = decode_lane_neon::<URL_SAFE_ALPHABET>(
            c0,
            hi_mask_tbl,
            lo_mask_tbl,
            roll_tbl,
            mask_0f,
            special_ch,
            special_adj,
        );
        let (chk1, v1) = decode_lane_neon::<URL_SAFE_ALPHABET>(
            c1,
            hi_mask_tbl,
            lo_mask_tbl,
            roll_tbl,
            mask_0f,
            special_ch,
            special_adj,
        );
        let (chk2, v2) = decode_lane_neon::<URL_SAFE_ALPHABET>(
            c2,
            hi_mask_tbl,
            lo_mask_tbl,
            roll_tbl,
            mask_0f,
            special_ch,
            special_adj,
        );
        let (chk3, v3) = decode_lane_neon::<URL_SAFE_ALPHABET>(
            c3,
            hi_mask_tbl,
            lo_mask_tbl,
            roll_tbl,
            mask_0f,
            special_ch,
            special_adj,
        );

        let min_chk = vminq_u8(vminq_u8(chk0, chk1), vminq_u8(chk2, chk3));
        if vminvq_u8(min_chk) == 0 {
            return None;
        }

        let out0 = vorrq_u8(vshlq_n_u8::<2>(v0), vshrq_n_u8::<4>(v1));
        let out1 = vorrq_u8(vshlq_n_u8::<4>(v1), vshrq_n_u8::<2>(v2));
        let out2 = vorrq_u8(vshlq_n_u8::<6>(v2), v3);
        Some(uint8x16x3_t(out0, out1, out2))
    }
}
