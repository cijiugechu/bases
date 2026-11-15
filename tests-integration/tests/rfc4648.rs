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

const VECTORS: &[(&[u8], &str, &str, &str, &str)] = &[
    (b"", "", "", "", ""),
    (b"f", "66", "MY======", "CO======", "Zg=="),
    (b"fo", "666F", "MZXQ====", "CPNG====", "Zm8="),
    (b"foo", "666F6F", "MZXW6===", "CPNMU===", "Zm9v"),
    (b"foob", "666F6F62", "MZXW6YQ=", "CPNMUOG=", "Zm9vYg=="),
    (b"fooba", "666F6F6261", "MZXW6YTB", "CPNMUOJ1", "Zm9vYmE="),
    (
        b"foobar",
        "666F6F626172",
        "MZXW6YTBOI======",
        "CPNMUOJ1E8======",
        "Zm9vYmFy",
    ),
];

#[test]
fn rfc_4648_vectors() {
    for &(input, base16, base32, base32hex, base64) in VECTORS {
        assert_vector::<Base16Upper>(input, base16);
        assert_vector::<Base16Lower>(input, &base16.to_ascii_lowercase());
        assert_vector::<Base32>(input, base32);
        assert_vector::<Base32Unpadded>(input, base32.trim_end_matches('='));
        assert_vector::<Base32Hex>(input, base32hex);
        assert_vector::<Base32HexUnpadded>(input, base32hex.trim_end_matches('='));
        assert_vector::<Base64>(input, base64);
        assert_vector::<Base64Unpadded>(input, base64.trim_end_matches('='));
    }
}

#[test]
fn base64url_alphabet() {
    assert_vector::<Base64Url>(&[0xfb, 0xff], "-_8=");
    assert_vector::<Base64UrlUnpadded>(&[0xfb, 0xff], "-_8");
}

#[test]
fn base16_decoding_is_case_insensitive() {
    let mut output = [0_u8; 2];

    assert_eq!(
        Base16Upper::decode_into(b"dead", &mut output).unwrap(),
        [0xde, 0xad]
    );
    assert_eq!(
        Base16Lower::decode_into(b"DEAD", &mut output).unwrap(),
        [0xde, 0xad]
    );
}

#[test]
fn profile_names_are_stable() {
    assert_eq!(Base16Lower::name(), "base16-lower");
    assert_eq!(Base16Upper::name(), "base16-upper");
    assert_eq!(Base32::name(), "base32");
    assert_eq!(Base32Unpadded::name(), "base32-unpadded");
    assert_eq!(Base32Hex::name(), "base32hex");
    assert_eq!(Base32HexUnpadded::name(), "base32hex-unpadded");
    assert_eq!(Base64::name(), "base64");
    assert_eq!(Base64Unpadded::name(), "base64-unpadded");
    assert_eq!(Base64Url::name(), "base64url");
    assert_eq!(Base64UrlUnpadded::name(), "base64url-unpadded");
}

fn assert_vector<C: bases::Codec>(input: &[u8], expected: &str) {
    let mut encoded_buffer = vec![0; expected.len()];
    let encoded: &mut str = C::encode_into(input, &mut encoded_buffer).unwrap();
    assert_eq!(encoded, expected, "encoding with {}", C::name());
    assert_eq!(C::encode(input), expected);

    let mut decoded_buffer = vec![0; C::max_decoded_len(expected.len())];
    let decoded: &mut [u8] = C::decode_into(expected, &mut decoded_buffer).unwrap();
    assert_eq!(decoded, input, "decoding with {}", C::name());
    assert_eq!(C::decode(expected).unwrap(), input);
}

#[test]
fn decoded_output_outlives_borrowed_and_owned_input() {
    let mut output = [0_u8; 1];
    let decoded = {
        let text = String::from("Zg==");
        Base64::decode_into(text.as_str(), &mut output).unwrap()
    };
    assert_eq!(decoded, b"f");

    let decoded = Base64::decode_into(String::from("Zw=="), &mut output).unwrap();
    assert_eq!(decoded, b"g");
}
