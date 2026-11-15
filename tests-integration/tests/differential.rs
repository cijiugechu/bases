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

use bases::Base16Lower;
use bases::Base16Upper;
use bases::Base32;
use bases::Base32Hex;
use bases::Base32HexUnpadded;
use bases::Base32Unpadded;
use bases::Base64;
use bases::Base64Unpadded;
use bases::Base64Url;
use bases::Base64UrlUnpadded;
use bases::Codec;
use data_encoding::BASE32 as REFERENCE_BASE32;
use data_encoding::BASE32_NOPAD as REFERENCE_BASE32_UNPADDED;
use data_encoding::BASE32HEX as REFERENCE_BASE32HEX;
use data_encoding::BASE32HEX_NOPAD as REFERENCE_BASE32HEX_UNPADDED;
use data_encoding::BASE64 as REFERENCE_BASE64;
use data_encoding::BASE64_NOPAD as REFERENCE_BASE64_UNPADDED;
use data_encoding::BASE64URL as REFERENCE_BASE64URL;
use data_encoding::BASE64URL_NOPAD as REFERENCE_BASE64URL_UNPADDED;
use data_encoding::Encoding;
use data_encoding::HEXLOWER as REFERENCE_HEXLOWER;
use data_encoding::HEXUPPER as REFERENCE_HEXUPPER;

#[test]
fn valid_inputs_match_data_encoding() {
    let mut input = [0_u8; 1024];

    for length in 0..=input.len() {
        fill_input(&mut input[..length], length);
        let input = &input[..length];

        assert_matches::<Base16Upper>(input, &REFERENCE_HEXUPPER);
        assert_matches::<Base16Lower>(input, &REFERENCE_HEXLOWER);
        assert_matches::<Base32>(input, &REFERENCE_BASE32);
        assert_matches::<Base32Unpadded>(input, &REFERENCE_BASE32_UNPADDED);
        assert_matches::<Base32Hex>(input, &REFERENCE_BASE32HEX);
        assert_matches::<Base32HexUnpadded>(input, &REFERENCE_BASE32HEX_UNPADDED);
        assert_matches::<Base64>(input, &REFERENCE_BASE64);
        assert_matches::<Base64Unpadded>(input, &REFERENCE_BASE64_UNPADDED);
        assert_matches::<Base64Url>(input, &REFERENCE_BASE64URL);
        assert_matches::<Base64UrlUnpadded>(input, &REFERENCE_BASE64URL_UNPADDED);
    }
}

fn assert_matches<C: Codec>(input: &[u8], reference: &Encoding) {
    let expected = reference.encode(input);
    let mut encoded_buffer = [0_u8; 2048];
    let encoded = C::encode_into(input, &mut encoded_buffer).unwrap();
    assert_eq!(
        &*encoded,
        expected,
        "encoding with {} at length {}",
        C::name(),
        input.len()
    );

    let mut decoded_buffer = [0_u8; 1024];
    let decoded = C::decode_into(expected.as_bytes(), &mut decoded_buffer).unwrap();
    assert_eq!(
        decoded,
        input,
        "decoding with {} at length {}",
        C::name(),
        input.len()
    );
}

fn fill_input(input: &mut [u8], seed: usize) {
    let mut state = seed as u64 ^ 0x9e37_79b9_7f4a_7c15;
    for byte in input {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *byte = state as u8;
    }
}
