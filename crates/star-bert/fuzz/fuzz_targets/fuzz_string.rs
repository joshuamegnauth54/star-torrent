#![no_main]

use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};
use star_bert::parser::{bytes, bytes_str};
use std::mem::size_of;

#[derive(Debug, Arbitrary)]
struct BertBytes<'bytes> {
    len: usize,
    delimiter: Option<char>,
    bytes: &'bytes [u8],
}

fuzz_target!(|data: &BertBytes| {
    let mut bencode: Vec<u8> = Vec::with_capacity(size_of::<BertBytes>() + data.bytes.len());
    bencode.extend(format!("{}{}", data.len, data.delimiter.unwrap_or_default()).as_bytes());

    let _ = bytes(&bencode);
});
