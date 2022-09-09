use std::num::NonZeroU64;

use serde::de::Unexpected;
use serde_bencode::{value::Value, Error as BencodeError};
use thiserror::Error;

/// Torrent serializing and deserializing errors.
#[derive(Debug, Error)]
pub enum ParseTorrentError {
    #[error("Ambiguous info dict (w.r.t. files): {0}")]
    AmbiguousFiles(&'static str),
    #[error("Bencode serializing/deserializing: {0}")]
    Bencode(#[from] BencodeError),
    #[error("Invalid torrent metainfo version: {0}")]
    InvalidVersion(u8),
    #[error("Piece length too small: {0}")]
    PieceLength(NonZeroU64),
}

#[inline]
pub(crate) fn value_to_unexpected(value: &Value) -> Unexpected {
    match value {
        Value::Int(i) => Unexpected::Signed(*i),
        Value::List(_) => Unexpected::Seq,
        Value::Bytes(bytes) => Unexpected::Bytes(bytes.as_slice()),
        Value::Dict(dict) => Unexpected::Map,
    }
}
