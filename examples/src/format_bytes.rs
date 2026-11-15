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

//! Format packet diagnostics without allocating an intermediate encoded string.

use std::fmt;
use std::io;
use std::io::Write;

use bases::Base64;
use bases::HexDisplay;

struct Packet<'a> {
    id: &'a [u8],
    payload: &'a [u8],
}

impl fmt::Display for Packet<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "packet_id={:x} payload={}",
            HexDisplay(self.id),
            Base64::display(self.payload),
        )
    }
}

fn main() -> io::Result<()> {
    let packet = Packet {
        id: b"\x01\x23\xab\xcd",
        payload: b"\x00\x01\xfe\xff",
    };

    let mut output = io::stdout().lock();
    writeln!(output, "{packet}")?;
    writeln!(output, "uppercase packet ID: {:X}", HexDisplay(packet.id))?;

    Ok(())
}
