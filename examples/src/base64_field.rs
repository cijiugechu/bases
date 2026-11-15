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

//! Carry a small binary payload in a JSON string field using padded Base64.
//!
//! JSON parsing is left to the application; decoding operates on the field value.

use std::error::Error;

use bases::Base64;
use bases::DecodeError;

fn main() -> Result<(), Box<dyn Error>> {
    let payload = b"\x00\x01\xfe\xff";
    let field = Base64::encode(payload);

    // Base64 contains neither quotes nor backslashes, so this field needs no JSON escaping.
    println!(r#"{{"payload":"{field}"}}"#);

    let restored = Base64::decode(&field)?;
    assert_eq!(restored, payload);
    println!("decoded payload: {restored:02x?}");

    match Base64::decode("AAH+/w?=") {
        Err(DecodeError::InvalidByte { index, byte }) => {
            println!("rejected payload: byte 0x{byte:02x} at offset {index}");
        }
        Err(error) => return Err(error.into()),
        Ok(_) => return Err("the malformed payload was unexpectedly accepted".into()),
    }

    Ok(())
}
