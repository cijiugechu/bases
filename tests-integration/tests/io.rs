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

use std::io;
use std::io::Read;
use std::io::Write;

use bases::Base16Lower;
use bases::Base32;
use bases::Base32Unpadded;
use bases::Base64;
use bases::Base64UrlUnpadded;
use bases::Codec;
use bases::DecodeError;
use bases::io::DecoderReader;
use bases::io::EncoderWriter;

struct ChunkedWriter {
    inner: Vec<u8>,
    max_write: usize,
    interrupt_every: usize,
    calls: usize,
}

impl ChunkedWriter {
    fn new(max_write: usize, interrupt_every: usize) -> Self {
        Self {
            inner: Vec::new(),
            max_write,
            interrupt_every,
            calls: 0,
        }
    }
}

impl Write for ChunkedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.calls += 1;
        if self.interrupt_every > 0 && self.calls.is_multiple_of(self.interrupt_every) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "interrupted"));
        }
        let take = buf.len().min(self.max_write);
        self.inner.extend_from_slice(&buf[..take]);
        Ok(take)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct ChunkedReader<'a> {
    data: &'a [u8],
    pos: usize,
    max_read: usize,
    interrupt_every: usize,
    calls: usize,
}

impl<'a> ChunkedReader<'a> {
    fn new(data: &'a [u8], max_read: usize, interrupt_every: usize) -> Self {
        Self {
            data,
            pos: 0,
            max_read,
            interrupt_every,
            calls: 0,
        }
    }
}

impl Read for ChunkedReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.calls += 1;
        if self.interrupt_every > 0 && self.calls.is_multiple_of(self.interrupt_every) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "interrupted"));
        }
        let remaining = self.data.len() - self.pos;
        let take = remaining.min(buf.len()).min(self.max_read);
        buf[..take].copy_from_slice(&self.data[self.pos..self.pos + take]);
        self.pos += take;
        Ok(take)
    }
}

#[test]
fn encoder_writer_and_decoder_reader_roundtrip_across_chunk_sizes() {
    fn check<C: Codec>(input: &[u8], write_chunk: usize, read_chunk: usize) {
        let mut writer = EncoderWriter::<C, _>::new(ChunkedWriter::new(write_chunk, 3));
        for chunk in input.chunks(write_chunk) {
            writer.write_all(chunk).unwrap();
        }
        writer.flush().unwrap();
        let inner = writer.finish().unwrap();
        let expected_encoded = C::encode(input);
        assert_eq!(inner.inner, expected_encoded.as_bytes());

        let mut reader = DecoderReader::<C, _>::new(ChunkedReader::new(&inner.inner, 1, 2));
        let mut decoded = Vec::new();
        let mut read_buf = vec![0_u8; read_chunk];
        loop {
            let n = reader.read(&mut read_buf).unwrap();
            if n == 0 {
                break;
            }
            decoded.extend_from_slice(&read_buf[..n]);
        }
        assert_eq!(decoded, input, "profile {}", C::name());
    }

    let payloads: [&[u8]; 6] = [
        b"",
        b"x",
        b"fo",
        b"foobar",
        &vec![0x5a_u8; 1009],
        &(0..1531_u16).map(|i| (i % 251) as u8).collect::<Vec<_>>(),
    ];

    for payload in payloads {
        for &(write_chunk, read_chunk) in &[(1, 1), (7, 3), (13, 5), (257, 17), (1009, 1024)] {
            check::<Base16Lower>(payload, write_chunk, read_chunk);
            check::<Base32>(payload, write_chunk, read_chunk);
            check::<Base32Unpadded>(payload, write_chunk, read_chunk);
            check::<Base64>(payload, write_chunk, read_chunk);
            check::<Base64UrlUnpadded>(payload, write_chunk, read_chunk);
        }
    }
}

#[test]
fn encoder_writer_handles_drop_and_into_inner() {
    // Inherent constructors work without Codec in scope.
    let mut dropped_out = Vec::new();
    {
        let mut writer = Base64::encoder_writer(&mut dropped_out);
        writer.write_all(b"hi").unwrap();
    }
    assert_eq!(dropped_out, b"aGk=");

    let mut dropped_empty = Vec::new();
    {
        let _writer = EncoderWriter::<Base32, _>::new(&mut dropped_empty);
    }
    assert!(dropped_empty.is_empty());

    let mut writer = Base64::encoder_writer(Vec::new());
    writer.write_all(b"f").unwrap();
    assert!(writer.get_ref().is_empty());
    let _ = writer.get_mut();
    let raw = writer.into_inner();
    assert!(raw.is_empty());
}

#[test]
fn decoder_reader_exposes_invalid_data_and_inner_accessors() {
    let cases: [(&[u8], DecodeError); 4] = [
        (b"Zg", DecodeError::InvalidLength { length: 2 }),
        (
            b"Zm9vYm!y",
            DecodeError::InvalidByte {
                index: 6,
                byte: b'!',
            },
        ),
        (b"Zg==AAAA", DecodeError::InvalidPadding { index: 2 }),
        (b"Zh==", DecodeError::NonCanonical { index: 1 }),
    ];

    for (wire, expected) in cases {
        let mut reader = Base64::decoder_reader(ChunkedReader::new(wire, 1, 0));
        assert_eq!(reader.get_ref().data, wire);
        let _ = reader.get_mut();
        let mut out = Vec::new();
        let err = reader.read_to_end(&mut out).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(
            err.get_ref().and_then(|e| e.downcast_ref::<DecodeError>()),
            Some(&expected)
        );
        assert_eq!(reader.into_inner().data, wire);
    }
}
