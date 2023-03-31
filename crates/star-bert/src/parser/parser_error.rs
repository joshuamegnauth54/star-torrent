use nom::error::{ContextError, ErrorKind, FromExternalError, ParseError};
use std::{num::ParseIntError, str::Utf8Error};
use thiserror::Error;

#[cfg(feature = "bigint")]
use num_bigint::ParseBigIntError;

#[derive(Debug, Error)]
#[error("{sources}")]
pub struct BertErrorTrace<I> {
    sources: Vec<BertError<I>>,
}

// Error traits from nom
// These are implemented similar to [nom::error::VerboseError]
impl<I> ContextError<I> for BertErrorTrace<I> {
    #[inline]
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other.sources.push(BertError {
            input,
            source: BertErrorKind::Context(ctx),
        });
        other
    }
}

impl<I> ParseError<I> for BertErrorTrace<I> {
    #[inline]
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            sources: vec![BertError {
                input,
                source: BertErrorKind::Nom(kind),
            }],
        }
    }

    #[inline]
    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other.sources.push(BertError {
            input,
            source: BertErrorKind::Nom(kind),
        });
        other
    }

    #[inline]
    fn from_char(input: I, ch: char) -> Self {
        Self {
            sources: vec![BertError {
                input,
                source: BertErrorKind::ErrantChar(ch),
            }],
        }
    }
}

impl<I> BertErrorTrace<I> {
    pub fn from_bert_error_kind(input: I, kind: BertErrorKind) -> Self {
        Self {
            sources: vec![
                BertError {
                    input,
                    source: kind
                }
            ]
        }
    }

}

impl<I, E> FromExternalError<I, E> for BertErrorTrace<I>
where
    E: Into<BertErrorKind>,
    I: Clone,
{
    fn from_external_error(input: I, kind: ErrorKind, e: E) -> Self {
        Self {
            sources: vec![
                BertError {
                    input: input.clone(),
                    source: BertErrorKind::Nom(kind),
                },
                BertError {
                    input,
                    source: e.into(),
                },
            ],
        }
    }
}

impl From<BertErrorTrace<&[u8]>> for BertErrorTrace<Vec<u8>> {
    #[inline]
    fn from(value: BertErrorTrace<&[u8]>) -> Self {
        Self {
            sources: value.sources.iter().map(From::from).collect(),
        }
    }
}

impl From<&BertErrorTrace<&[u8]>> for BertErrorTrace<Vec<u8>> {
    #[inline]
    fn from(value: &BertErrorTrace<&[u8]>) -> Self {
        Self {
            sources: value.sources.iter().map(From::from).collect(),
        }
    }
}

/*impl<I> CloneToVec<Vec<u8>> for BertErrorTrace<I>
where
    I: Borrow<[u8]>,
{
    type InputOwned = BertErrorTrace<Vec<u8>>;

    #[inline]
    fn clone_to_vec(&self) -> Self::InputOwned {
        BertErrorTrace {
            sources: self.sources.iter().map(CloneToVec::<I>::clone_to_vec).collect(),
        }
    }
}*/

#[derive(Debug, Error)]
#[error("{source}")]
pub struct BertError<I> {
    /// Errant input
    input: I,
    /// The kind of error that occurred
    source: BertErrorKind,
}

/*impl<I> CloneToVec<Vec<u8>> for BertError<I>
where
    I: Clone + ToOwned<Owned = Vec<u8>>,
{
    type InputOwned = BertError<Vec<u8>>;

    #[inline]
    fn clone_to_vec(&self) -> Self::InputOwned {
        BertError {
            input: self.input.to_owned(),
            source: self.source.clone(),
        }
    }
}*/

impl From<BertError<&[u8]>> for BertError<Vec<u8>> {
    #[inline]
    fn from(value: BertError<&[u8]>) -> Self {
        Self {
            input: value.input.to_owned(),
            source: value.source,
        }
    }
}

impl From<&BertError<&[u8]>> for BertError<Vec<u8>> {
    /// Essentially like a [std::borrow::ToOwned] implementation because I can't figure it out.
    #[inline]
    fn from(value: &BertError<&[u8]>) -> Self {
        Self {
            input: value.input.to_owned(),
            source: value.source.clone(),
        }
    }
}

#[derive(Debug, Error, Clone)]
pub enum BertErrorKind {
    #[error("parsing integer from bencode: {0}")]
    ParseInt(#[from] ParseIntegerDelegate),
    #[error("expected valid UTF-8: {0}")]
    Unicode(#[from] Utf8Error),
    #[error("context: {0}")]
    Context(&'static str),
    #[error("nom: {0:?}")]
    Nom(ErrorKind),
    #[error("unexpected char: {0}")]
    ErrantChar(char),
}

impl From<ParseIntError> for BertErrorKind {
    #[inline]
    fn from(value: ParseIntError) -> Self {
        let int_any: ParseIntegerAnyError = value.into();
        let delegate: ParseIntegerDelegate = int_any.into();
        delegate.into()
    }
}

#[cfg(feature = "bigint")]
impl From<ParseBigIntError> for BertErrorKind {
    #[inline]
    fn from(value: ParseBigIntError) -> Self {
        let int_any: ParseIntegerAnyError = value.into();
        let delegate: ParseIntegerDelegate = int_any.into();
        delegate.into()
    }
}

/// Wrapper around the standard library's [ParseIntError] and the number crate's [num_bigint::ParseBigIntError] so
/// that [BencodeErrorKind] only has one variant for errors parsing integers.
#[derive(Debug, Error, Clone)]
#[non_exhaustive]
#[error(transparent)]
enum ParseIntegerAnyError {
    ParseIntStd(#[from] ParseIntError),
    #[cfg(feature = "bigint")]
    ParseBigInt(#[from] ParseBigIntError),
    // #[cfg(not(bigint))]
    // ParseBigInt(ParseBigIntEmpty),
}

/// Wrapper around a integer parsing errors.
/// [ParseIntegerDelegate] is primarily to avoid leaking an implementation detail for [BencodeErrorKind].
#[derive(Debug, Error, Clone)]
#[error(transparent)]
pub struct ParseIntegerDelegate(#[from] ParseIntegerAnyError);
