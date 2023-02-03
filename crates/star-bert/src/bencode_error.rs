use std::{num::ParseIntError, str::Utf8Error};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{source}")]
pub struct BencodeError {
    /// Errant bytes
    bytes: Option<Vec<u8>>,
    /// The kind of error that occurred
    source: BencodeErrorKind,
}

impl BencodeError {
    /// Construct [BencodeError] from bytes that caused an error as well as the error kind ([BencodeErrorKind]).
    #[inline]
    pub(crate) fn from_bytes_source<E>(bytes: Option<&[u8]>, source: E) -> Self
    where
        E: Into<BencodeErrorKind>,
    {
        Self {
            bytes: bytes.map(ToOwned::to_owned),
            source: source.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum BencodeErrorKind {
    #[error("parsing i64 from bencode: {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("expected valid UTF-8: {0}")]
    Unicode(#[from] Utf8Error),
}
