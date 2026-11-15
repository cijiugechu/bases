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

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;

use bases::Base16Lower;
use bases::Base16Upper;
use bases::Base32;
use bases::Base32Hex;
use bases::Base32HexUnpadded;
use bases::Base32Unpadded;
use bases::Base64;
use bases::Base64UrlUnpadded;
use bases::DecodeError;
use bases::Decoder;
use bases::EncodeError;
use bases::EncodedDisplay;
use bases::Encoder;
use bases::base16;
use bases::base32;
use bases::base64;
use bases::policy;

type Base16LowerStrict = base16::Base16<policy::Lower, policy::Strict>;
type Base16UpperStrict = base16::Base16<policy::Upper, policy::Strict>;
type Base32HexLowerMixedUnpadded =
    base32::Base32<base32::ExtendedHex, policy::Unpadded, policy::Lower, policy::Mixed>;
type Base32UpperMixedUnpadded =
    base32::Base32<base32::Standard, policy::Unpadded, policy::Upper, policy::Mixed>;
type Base32LowerStrictPadded =
    base32::Base32<base32::Standard, policy::Padded, policy::Lower, policy::Strict>;

const CONST_NAME_BASE16_STRICT: &str = Base16LowerStrict::name();
const CONST_NAME_RFC5155: &str = Base32HexLowerMixedUnpadded::name();
const CONST_ENCODED_LEN: usize = match Base64::encoded_len(5) {
    Ok(len) => len,
    Err(_) => panic!("encoded length fits in usize"),
};
const CONST_OVERFLOW: Result<usize, EncodeError> = Base16Upper::encoded_len(usize::MAX);
const CONST_MAX_DECODED_LEN: usize = Base32HexUnpadded::max_decoded_len(16);
const CONST_DISPLAY: EncodedDisplay<'static, Base64UrlUnpadded> = Base64UrlUnpadded::display(b"hi");
const CONST_ENCODER: Encoder<Base32> = Base32::encoder();
const CONST_DECODER: Decoder<Base16Lower> = Base16Lower::decoder();
const CONST_PROFILE: Base16LowerStrict = base16::Base16::new();

#[test]
fn const_fn_inherent_apis_evaluate_at_compile_time() {
    assert_eq!(CONST_NAME_BASE16_STRICT, "base16-lower-strict");
    assert_eq!(CONST_NAME_RFC5155, "base32hex-lower-mixed-unpadded");
    assert_eq!(CONST_ENCODED_LEN, 8);
    assert_eq!(CONST_OVERFLOW, Err(EncodeError::LengthOverflow));
    assert_eq!(CONST_MAX_DECODED_LEN, 10);
    assert_eq!(CONST_DISPLAY.to_string(), "aGk");

    let mut encoded = [0_u8; CONST_ENCODED_LEN];
    let text = Base64::encode_into(b"hello", &mut encoded).unwrap();
    assert_eq!(text, "aGVsbG8=");

    let mut enc = CONST_ENCODER;
    let mut enc_out = [0_u8; 8];
    assert_eq!(enc.update(b"fo", &mut enc_out).unwrap(), "");
    assert_eq!(enc.finish(&mut enc_out).unwrap(), "MZXQ====");

    let mut dec = CONST_DECODER;
    let mut dec_out = [0_u8; 4];
    assert_eq!(dec.update(b"DeAd", &mut dec_out).unwrap(), [0xde, 0xad]);
    assert_eq!(dec.finish(&mut dec_out).unwrap(), b"");

    let profile_copy = CONST_PROFILE;
    assert_eq!(profile_copy, Base16LowerStrict::default());
    assert!(!format!("{profile_copy:?}").is_empty());

    let mut hasher = DefaultHasher::new();
    profile_copy.hash(&mut hasher);
}

#[test]
fn base16_strict_case_policies_enforce_case_across_simd_boundaries() {
    assert_eq!(Base16UpperStrict::name(), "base16-upper-strict");
    assert_eq!(Base16LowerStrict::name(), "base16-lower-strict");

    for len in 0..=130 {
        let input: Vec<u8> = (0..len).map(|i| (i * 37 + 11) as u8).collect();
        let expected_lower = data_encoding::HEXLOWER.encode(&input);
        let expected_upper = data_encoding::HEXUPPER.encode(&input);

        assert_eq!(Base16LowerStrict::encode(&input), expected_lower);
        assert_eq!(Base16UpperStrict::encode(&input), expected_upper);

        assert_eq!(Base16LowerStrict::decode(&expected_lower).unwrap(), input);
        assert_eq!(Base16UpperStrict::decode(&expected_upper).unwrap(), input);

        if len > 0 {
            for pos in [0, 1, 15, 16, 17, 31, 32, 63, 64, expected_lower.len() - 1]
                .into_iter()
                .filter(|&p| p < expected_lower.len())
            {
                if expected_lower.as_bytes()[pos].is_ascii_lowercase() {
                    let mut flipped = expected_lower.clone().into_bytes();
                    flipped[pos] = flipped[pos].to_ascii_uppercase();
                    assert_eq!(
                        Base16LowerStrict::decode(&flipped),
                        Err(DecodeError::InvalidByte {
                            index: pos,
                            byte: flipped[pos],
                        })
                    );
                    assert_eq!(Base16Lower::decode(&flipped).unwrap(), input);
                }
                if expected_upper.as_bytes()[pos].is_ascii_uppercase() {
                    let mut flipped = expected_upper.clone().into_bytes();
                    flipped[pos] = flipped[pos].to_ascii_lowercase();
                    assert_eq!(
                        Base16UpperStrict::decode(&flipped),
                        Err(DecodeError::InvalidByte {
                            index: pos,
                            byte: flipped[pos],
                        })
                    );
                    assert_eq!(Base16Upper::decode(&flipped).unwrap(), input);
                }
            }
        }
    }
}

#[test]
fn base32_extended_hex_lower_mixed_unpadded_matches_rfc5155() {
    // RFC 5155 Appendix A NSEC3 hashes (H(example) and H(ns1.example)).
    for nsec3_hash in [
        "0p9mhaveqvm6t7vbl5lop2u3t2rp3tom",
        "2t7b4g4vsa5smi47k61mv5bv1a22bojr",
    ] {
        let raw_digest = data_encoding::BASE32HEX_NOPAD
            .decode(nsec3_hash.to_ascii_uppercase().as_bytes())
            .expect("valid RFC 5155 Base32Hex vector");
        let encoded = Base32HexLowerMixedUnpadded::encode(&raw_digest);
        assert_eq!(encoded, nsec3_hash);
        assert_eq!(
            Base32HexLowerMixedUnpadded::decode(encoded.to_ascii_uppercase()).unwrap(),
            raw_digest
        );
    }

    for len in 0..=160 {
        let input: Vec<u8> = (0..len).map(|i| (i * 53 + 7) as u8).collect();
        let expected = data_encoding::BASE32HEX_NOPAD
            .encode(&input)
            .to_ascii_lowercase();
        let actual = Base32HexLowerMixedUnpadded::encode(&input);
        assert_eq!(actual, expected);
        assert_eq!(Base32HexLowerMixedUnpadded::decode(&actual).unwrap(), input);

        // Mixed upper/lower input must decode identically across scalar and SIMD lanes.
        let mixed: String = actual
            .bytes()
            .enumerate()
            .map(|(idx, b)| {
                if idx % 2 == 0 {
                    b.to_ascii_uppercase() as char
                } else {
                    b as char
                }
            })
            .collect();
        assert_eq!(Base32HexLowerMixedUnpadded::decode(&mixed).unwrap(), input);

        // Standard-only Base32 symbols such as 'w'/'z' remain rejected.
        assert_eq!(
            Base32HexLowerMixedUnpadded::decode("0000000z"),
            Err(DecodeError::InvalidByte {
                index: 7,
                byte: b'z',
            })
        );
    }
}

#[test]
fn base32_standard_case_and_padding_policies() {
    assert_eq!(Base32UpperMixedUnpadded::name(), "base32-mixed-unpadded");
    assert_eq!(Base32LowerStrictPadded::name(), "base32-lower");

    for len in 0..=160 {
        let input: Vec<u8> = (0..len).map(|i| (i * 29 + 3) as u8).collect();
        let upper_unpadded = data_encoding::BASE32_NOPAD.encode(&input);
        assert_eq!(Base32UpperMixedUnpadded::encode(&input), upper_unpadded);

        let mixed_unpadded: String = upper_unpadded
            .bytes()
            .enumerate()
            .map(|(idx, b)| {
                if idx % 3 == 1 {
                    b.to_ascii_lowercase() as char
                } else {
                    b as char
                }
            })
            .collect();
        assert_eq!(
            Base32UpperMixedUnpadded::decode(&mixed_unpadded).unwrap(),
            input
        );

        let lower_padded = data_encoding::BASE32.encode(&input).to_ascii_lowercase();
        assert_eq!(Base32LowerStrictPadded::encode(&input), lower_padded);
        assert_eq!(
            Base32LowerStrictPadded::decode(&lower_padded).unwrap(),
            input
        );
    }

    assert_eq!(
        Base32UpperMixedUnpadded::decode("my======"),
        Err(DecodeError::InvalidPadding { index: 2 })
    );
    assert_eq!(
        Base32UpperMixedUnpadded::decode("m0"),
        Err(DecodeError::InvalidByte {
            index: 1,
            byte: b'0',
        })
    );
    assert_eq!(
        Base32LowerStrictPadded::decode("MY======"),
        Err(DecodeError::InvalidByte {
            index: 0,
            byte: b'M',
        })
    );
    assert_eq!(
        Base32Hex::name(),
        base32::Base32::<base32::ExtendedHex, policy::Padded, policy::Upper, policy::Strict>::name(
        )
    );
    assert_eq!(
        Base32Unpadded::name(),
        base32::Base32::<base32::Standard, policy::Unpadded, policy::Upper, policy::Strict>::name()
    );
    assert_eq!(
        base64::Base64::<base64::Standard, policy::Padded>::name(),
        "base64"
    );
    assert_eq!(
        base64::Base64::<base64::UrlSafe, policy::Unpadded>::name(),
        "base64url-unpadded"
    );
}
