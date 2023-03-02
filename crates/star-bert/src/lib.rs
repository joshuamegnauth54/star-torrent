//! BERT Encodes Rusty Torrents (BERT) is a [Bencode](http://bittorrent.org/beps/bep_0003.html) parser with an optional Serde implementation.
//!
//!
//! ## Type parameter conventions
//! - `I`: Input, such as `&`[u8]
//! - `N`: Numbers that implement [num_integer::Integer]

pub mod parser;
