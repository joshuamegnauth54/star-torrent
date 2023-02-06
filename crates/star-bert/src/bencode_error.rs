use std::{num::ParseIntError, str::Utf8Error};
use thiserror::Error;

#[cfg(bigint)]
use num_bigint::ParseBigIntError;

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
    #[error("parsing integer from bencode: {0}")]
    ParseInt(#[from] ParseIntegerDelegate),
    #[error("expected valid UTF-8: {0}")]
    Unicode(#[from] Utf8Error),
}

impl From<ParseIntError> for BencodeErrorKind {
    #[inline]
    fn from(value: ParseIntError) -> Self {
        let int_any: ParseIntegerAnyError = value.into();
        let delegate: ParseIntegerDelegate = int_any.into();
        delegate.into()
    }
}

#[cfg(bigint)]
impl From<ParseBigIntError> for BencodeErrorKind {
    #[inline]
    fn from(value: ParseBigIntError) -> Self {
        let int_any: ParseIntegerAnyError = value.into();
        let delegate: ParseIntegerDelegate = int_any.into();
        delegate.into()
    }
}

/// Wrapper around the standard library's [ParseIntError] and the number crate's [num_bigint::ParseBigIntError] so
/// that [BencodeErrorKind] only has one variant for errors parsing integers.
#[derive(Debug, Error)]
#[non_exhaustive]
#[error(transparent)]
enum ParseIntegerAnyError {
    ParseIntStd(#[from] ParseIntError),
    #[cfg(bigint)]
    ParseBigInt(#[from] ParseBigIntError),
    // #[cfg(not(bigint))]
    // ParseBigInt(ParseBigIntEmpty),
}

/// Wrapper around a integer parsing errors.
/// [ParseIntegerDelegate] is primarily to avoid leaking an implementation detail for [BencodeErrorKind].
#[derive(Debug, Error)]
#[error(transparent)]
pub struct ParseIntegerDelegate(#[from] ParseIntegerAnyError);
