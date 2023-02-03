use crate::bencode_error::BencodeError;
use nom::{
    bytes::complete::tag, character::complete::digit1, combinator::map_res, sequence::delimited,
    IResult,
};

/// Parse a Bencoded integer from bytes.
///
/// Integers are always base ten numbers delimited with 'i' and 'e'. For example, the number 14 Bencoded is `i14e`.
/// Integers are signed; `i-28e` and `i-0e` are both valid.
/// Integers cannot start with a leading 0. `i042e` is invalid.
///
/// # Examples
///
/// A basic, positive integer.
///
/// ```rust
/// use star_bert::parser::integer;
/// use star_bert::bencode_error::BencodeError;
///
/// let num = integer(b"i14e")?;
/// assert_eq!(num, 14);
///
/// # Ok::<(), BencodeError>(())
/// ```
///
/// Negative integer
/// ```rust
/// use star_bert::parser::integer;
/// use star_bert::bencode_error::BencodeError;
///
/// let num = integer(b"i-28e")?;
/// assert_eq!(num, -28);
///
/// # Ok::<(), BencodeError>(());
/// ```
///
#[inline]
pub fn integer(input: &[u8]) -> IResult<&[u8], i64> {
    map_res(delimited(tag("i"), digit1, tag("e")), bytes_to_str_to_i64)(input)
}

// Helper functions
/// Parse a [u8] slice to [str] and then [i64].
#[inline]
fn bytes_to_str_to_i64(bytes: &[u8]) -> Result<i64, BencodeError> {
    std::str::from_utf8(bytes)
        .map_err(|source| BencodeError::from_bytes_source(Some(bytes), source))
        .and_then(|s| {
            s.parse()
                .map_err(|source| BencodeError::from_bytes_source(Some(bytes), source))
        })
}
