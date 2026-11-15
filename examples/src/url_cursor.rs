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

//! Encode a pagination cursor for a query parameter with unpadded Base64URL.
//!
//! The cursor contains one big-endian record ID; its binary layout is application-defined.

use std::error::Error;

use bases::Base64UrlUnpadded;

fn main() -> Result<(), Box<dyn Error>> {
    let last_record_id = 42_u64;
    let cursor = Base64UrlUnpadded::encode(&last_record_id.to_be_bytes());
    println!("next page: /records?after={cursor}");

    let id_bytes = Base64UrlUnpadded::decode_array::<8>(&cursor)?;
    let after_id = u64::from_be_bytes(id_bytes);

    assert_eq!(after_id, last_record_id);
    println!("resume after record {after_id}");

    Ok(())
}
