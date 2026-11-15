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

//! Convert a batch using output buffers sized once for the largest record.

use std::error::Error;

use bases::Base16Upper;
use bases::Base32Unpadded;
use bases::Base64;
use bases::Codec;

fn main() -> Result<(), Box<dyn Error>> {
    let records: &[&[u8]] = &[b"\x01\x00\x14\xff", b"\x02\x01", b"\x03\x00\x1a"];

    convert_batch::<Base16Upper>(records)?;
    convert_batch::<Base32Unpadded>(records)?;
    convert_batch::<Base64>(records)?;

    Ok(())
}

fn convert_batch<C: Codec>(records: &[&[u8]]) -> Result<(), Box<dyn Error>> {
    let max_input_len = records.iter().map(|record| record.len()).max().unwrap_or(0);
    let encoded_len = C::encoded_len(max_input_len)?;
    let decoded_len = C::max_decoded_len(encoded_len);
    let mut encoded_buffer = vec![0_u8; encoded_len];
    let mut decoded_buffer = vec![0_u8; decoded_len];

    for &record in records {
        let encoded = C::encode_into(record, &mut encoded_buffer)?;
        let decoded = C::decode_into(&*encoded, &mut decoded_buffer)?;

        // Only the returned prefixes belong to this record; the buffers may contain older data.
        assert_eq!(decoded, record);
        println!("{}: {} bytes -> {encoded}", C::name(), decoded.len());
    }

    Ok(())
}
