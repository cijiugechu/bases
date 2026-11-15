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
use bases::Base64;
use bases::Base64UrlUnpadded;

fn main() {
    let input = b"hello, world";

    println!("Base16 (lower): {}", Base16Lower::display(input));
    println!("Base16 (upper): {}", Base16Upper::display(input));
    println!("Base32: {}", Base32::display(input));
    println!("Base64: {}", Base64::display(input));
    println!(
        "Base64URL (unpadded): {}",
        Base64UrlUnpadded::display(input)
    );
}
