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

//! AArch64 NEON Base16 kernels.

use core::arch::aarch64::uint8x16_t;
use core::arch::aarch64::uint8x16x2_t;
use core::arch::aarch64::vaddq_u8;
use core::arch::aarch64::vandq_u8;
use core::arch::aarch64::vdupq_n_u8;
use core::arch::aarch64::vld1q_u8;
use core::arch::aarch64::vld2q_u8;
use core::arch::aarch64::vminq_u8;
use core::arch::aarch64::vminvq_u8;
use core::arch::aarch64::vqtbl1q_u8;
use core::arch::aarch64::vshrq_n_u8;
use core::arch::aarch64::vsliq_n_u8;
use core::arch::aarch64::vst1q_u8;
use core::arch::aarch64::vst2q_u8;

const UPPER_HEX: [u8; 16] = *b"0123456789ABCDEF";
const LOWER_HEX: [u8; 16] = *b"0123456789abcdef";

const HI_MASK_MIXED: [u8; 16] = [0, 0, 0, 1, 2, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const HI_MASK_UPPER: [u8; 16] = [0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const HI_MASK_LOWER: [u8; 16] = [0, 0, 0, 1, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const LO_MASK: [u8; 16] = [1, 7, 7, 7, 7, 7, 7, 1, 1, 1, 0, 0, 0, 0, 0, 0];
const HI_OFFSET: [u8; 16] = [0, 0, 0, 0, 9, 0, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0];

const fn select_hi_mask<const UPPERCASE: bool, const STRICT_CASE: bool>() -> &'static [u8; 16] {
    if !STRICT_CASE {
        &HI_MASK_MIXED
    } else if UPPERCASE {
        &HI_MASK_UPPER
    } else {
        &HI_MASK_LOWER
    }
}

pub fn encode<const UPPERCASE: bool>(input: &[u8], output: &mut [u8]) -> (usize, usize) {
    let chunks = (input.len() / 16).min(output.len() / 32);
    if chunks == 0 {
        return (0, 0);
    }

    // SAFETY: `chunks * 16 <= input.len()` and `chunks * 32 <= output.len()`, and NEON is baseline
    // on aarch64.
    unsafe { encode_neon::<UPPERCASE>(input.as_ptr(), output.as_mut_ptr(), chunks) };
    (chunks * 16, chunks * 32)
}

pub fn decode<const UPPERCASE: bool, const STRICT_CASE: bool>(
    input: &[u8],
    output: &mut [u8],
) -> (usize, usize) {
    let chunks = (input.len() / 32).min(output.len() / 16);
    if chunks == 0 {
        return (0, 0);
    }

    // SAFETY: `chunks * 32 <= input.len()` and `chunks * 16 <= output.len()`, and NEON is baseline
    // on aarch64.
    let done = unsafe {
        decode_neon::<UPPERCASE, STRICT_CASE>(input.as_ptr(), output.as_mut_ptr(), chunks)
    };
    (done * 32, done * 16)
}

pub fn decode_in_place<const UPPERCASE: bool, const STRICT_CASE: bool>(
    buf: &mut [u8],
) -> (usize, usize) {
    let chunks = buf.len() / 32;
    if chunks == 0 {
        return (0, 0);
    }

    // SAFETY: `chunks * 32 <= buf.len()`, each iteration loads 32 bytes before storing 16 bytes,
    // and NEON is baseline on aarch64.
    let done = unsafe { decode_in_place_neon::<UPPERCASE, STRICT_CASE>(buf.as_mut_ptr(), chunks) };
    (done * 32, done * 16)
}

/// # Safety
///
/// Caller must ensure `in_ptr` is valid for `chunks * 16` reads and `out_ptr` is valid for
/// `chunks * 32` non-overlapping writes, with NEON enabled.
#[target_feature(enable = "neon")]
unsafe fn encode_neon<const UPPERCASE: bool>(
    mut in_ptr: *const u8,
    mut out_ptr: *mut u8,
    chunks: usize,
) {
    let table = if UPPERCASE { &UPPER_HEX } else { &LOWER_HEX };
    // SAFETY: Caller guarantees `chunks * 16` readable bytes at `in_ptr`, `chunks * 32` writable
    // bytes at `out_ptr`, and `table` is a 16-byte static array.
    unsafe {
        let hex_tbl = vld1q_u8(table.as_ptr());
        let mask_0f = vdupq_n_u8(0x0f);
        for _ in 0..chunks {
            let v = vld1q_u8(in_ptr);
            let hi = vshrq_n_u8::<4>(v);
            let lo = vandq_u8(v, mask_0f);
            let hi_ascii = vqtbl1q_u8(hex_tbl, hi);
            let lo_ascii = vqtbl1q_u8(hex_tbl, lo);
            vst2q_u8(out_ptr, uint8x16x2_t(hi_ascii, lo_ascii));
            in_ptr = in_ptr.add(16);
            out_ptr = out_ptr.add(32);
        }
    }
}

/// # Safety
///
/// Caller must ensure `in_ptr` is valid for `chunks * 32` reads and `out_ptr` is valid for
/// `chunks * 16` non-overlapping writes, with NEON enabled.
#[target_feature(enable = "neon")]
unsafe fn decode_neon<const UPPERCASE: bool, const STRICT_CASE: bool>(
    mut in_ptr: *const u8,
    mut out_ptr: *mut u8,
    chunks: usize,
) -> usize {
    let hi_mask_ptr = select_hi_mask::<UPPERCASE, STRICT_CASE>().as_ptr();
    let mut done = 0;
    // SAFETY: Caller guarantees `chunks * 32` readable bytes at `in_ptr`, `chunks * 16` writable
    // bytes at `out_ptr`, and table pointers point to 16-byte static arrays.
    unsafe {
        let hi_mask_tbl = vld1q_u8(hi_mask_ptr);
        let lo_mask_tbl = vld1q_u8(LO_MASK.as_ptr());
        let hi_off_tbl = vld1q_u8(HI_OFFSET.as_ptr());
        let mask_0f = vdupq_n_u8(0x0f);

        while done < chunks {
            let uint8x16x2_t(even, odd) = vld2q_u8(in_ptr);
            let Some(packed) =
                decode_pair_neon(even, odd, hi_mask_tbl, lo_mask_tbl, hi_off_tbl, mask_0f)
            else {
                break;
            };
            vst1q_u8(out_ptr, packed);
            in_ptr = in_ptr.add(32);
            out_ptr = out_ptr.add(16);
            done += 1;
        }
    }
    done
}

/// # Safety
///
/// Caller must ensure `ptr` is valid for `chunks * 32` reads and writes, with NEON enabled.
#[target_feature(enable = "neon")]
unsafe fn decode_in_place_neon<const UPPERCASE: bool, const STRICT_CASE: bool>(
    ptr: *mut u8,
    chunks: usize,
) -> usize {
    let hi_mask_ptr = select_hi_mask::<UPPERCASE, STRICT_CASE>().as_ptr();
    let mut in_ptr: *const u8 = ptr;
    let mut out_ptr: *mut u8 = ptr;
    let mut done = 0;
    // SAFETY: Caller guarantees `chunks * 32` readable/writable bytes from `ptr`. Each iteration
    // loads `in_ptr[0..32]` completely before writing `out_ptr[0..16]`, and `out_ptr` lags
    // behind `in_ptr` by at least `16 * done` bytes so future reads are never overwritten.
    unsafe {
        let hi_mask_tbl = vld1q_u8(hi_mask_ptr);
        let lo_mask_tbl = vld1q_u8(LO_MASK.as_ptr());
        let hi_off_tbl = vld1q_u8(HI_OFFSET.as_ptr());
        let mask_0f = vdupq_n_u8(0x0f);

        while done < chunks {
            let uint8x16x2_t(even, odd) = vld2q_u8(in_ptr);
            let Some(packed) =
                decode_pair_neon(even, odd, hi_mask_tbl, lo_mask_tbl, hi_off_tbl, mask_0f)
            else {
                break;
            };
            vst1q_u8(out_ptr, packed);
            in_ptr = in_ptr.add(32);
            out_ptr = out_ptr.add(16);
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
unsafe fn decode_pair_neon(
    even: uint8x16_t,
    odd: uint8x16_t,
    hi_mask_tbl: uint8x16_t,
    lo_mask_tbl: uint8x16_t,
    hi_off_tbl: uint8x16_t,
    mask_0f: uint8x16_t,
) -> Option<uint8x16_t> {
    let hi_even = vshrq_n_u8::<4>(even);
    let lo_even = vandq_u8(even, mask_0f);
    let hi_odd = vshrq_n_u8::<4>(odd);
    let lo_odd = vandq_u8(odd, mask_0f);

    let chk_even = vandq_u8(
        vqtbl1q_u8(hi_mask_tbl, hi_even),
        vqtbl1q_u8(lo_mask_tbl, lo_even),
    );
    let chk_odd = vandq_u8(
        vqtbl1q_u8(hi_mask_tbl, hi_odd),
        vqtbl1q_u8(lo_mask_tbl, lo_odd),
    );
    if vminvq_u8(vminq_u8(chk_even, chk_odd)) == 0 {
        return None;
    }

    let nib_even = vaddq_u8(lo_even, vqtbl1q_u8(hi_off_tbl, hi_even));
    let nib_odd = vaddq_u8(lo_odd, vqtbl1q_u8(hi_off_tbl, hi_odd));
    Some(vsliq_n_u8::<4>(nib_odd, nib_even))
}
