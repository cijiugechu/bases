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

//! Policy markers for encoding and decoding profiles.

/// Padding policy that requires and emits canonical `=` padding.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Padded;

/// Padding policy that omits and rejects `=` padding.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Unpadded;

/// Letter case policy that emits lowercase ASCII letters.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Lower;

/// Letter case policy that emits uppercase ASCII letters.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Upper;

/// Decoding case policy that accepts both uppercase and lowercase ASCII letters.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Mixed;

/// Decoding case policy that accepts only the configured letter case.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Strict;

impl sealed::PaddingPolicy for Padded {
    const PADDED: bool = true;
}

impl sealed::PaddingPolicy for Unpadded {
    const PADDED: bool = false;
}

impl sealed::LetterCase for Lower {
    const UPPERCASE: bool = false;
}

impl sealed::LetterCase for Upper {
    const UPPERCASE: bool = true;
}

impl sealed::DecodeCasePolicy for Mixed {
    const STRICT: bool = false;
}

impl sealed::DecodeCasePolicy for Strict {
    const STRICT: bool = true;
}

pub(crate) mod sealed {
    pub trait PaddingPolicy: 'static {
        const PADDED: bool;
    }

    pub trait LetterCase: 'static {
        const UPPERCASE: bool;
    }

    pub trait DecodeCasePolicy: 'static {
        const STRICT: bool;
    }
}
