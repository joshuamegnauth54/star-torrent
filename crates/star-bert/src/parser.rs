//! Recursive descent Bencode parser.

use crate::bencode_error::{BencodeError, BencodeErrorKind};
use nom::{
    bytes::complete::tag, character::complete::digit1, combinator::map_res, sequence::delimited,
    IResult,
};
use num_integer::Integer;
use std::{fmt::Debug, str::FromStr};

/// Parse a Bencoded integer from bytes.
///
/// Integers are always base ten numbers delimited with 'i' and 'e'. For example, the number 14 Bencoded is `i14e`.
/// Integers are signed; `i-28e` and `i-0e` are both valid.
/// Integers cannot start with a leading 0. `i042e` is invalid.
/// Per the spec: "Integers have no size limitation"
/// # Examples
///
/// A basic, positive integer.
///
/// ```rust
/// use star_bert::parser::integer;
/// use star_bert::bencode_error::BencodeError;
///
/// let (_bytes, num) = integer::<u8>(b"i14e")?;
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
/// let (_bytes, num) = integer::<i32>(b"i-28e")?;
/// assert_eq!(num, -28);
///
/// # Ok::<(), BencodeError>(());
/// ```
///
/// ## Arbitrarily sized integers (BigInt)
/// The Bencode spec doesn't define the size of integers. In other words, an int isn't a `i64` or any other type.
/// BigInts can be enabled via the `bigint` feature which is disabled by default.
///
/// ```rust
/// use star_bert::parser::integer;
/// use star_bert::bencode_error::BencodeError;
/// use num_bigint::BigInt;
///
/// let big_num = format!("i{}e", u128::MAX + 1);
/// let mun_gib = integer::<BigInt>(big_num.as_bytes())?;
///
/// # Ok::<(), BencodeError>(());
/// ```
#[inline]
pub fn integer<I>(input: &[u8]) -> IResult<&[u8], I>
where
    I: Integer + FromStr,
    <I as FromStr>::Err: Debug + Into<BencodeErrorKind>,
{
    map_res(delimited(tag("i"), digit1, tag("e")), bytes_to_str_to_int)(input)
}

// Helper functions
/// Parse a [u8] slice to [str] and then to impl [Integer].
#[inline]
fn bytes_to_str_to_int<I>(bytes: &[u8]) -> Result<I, BencodeError>
where
    I: Integer + FromStr,
    <I as FromStr>::Err: Debug + Into<BencodeErrorKind>,
{
    std::str::from_utf8(bytes)
        .map_err(|source| BencodeError::from_bytes_source(Some(bytes), source))
        .and_then(|s| {
            s.parse()
                .map_err(|source| BencodeError::from_bytes_source(Some(bytes), source))
        })
}

#[cfg(test)]
mod tests {
    use crate::{
        bencode_error::BencodeError,
        parser::{bytes_to_str_to_int, integer},
    };

    #[cfg(bigint)]
    use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};

    #[test]
    #[should_panic(expected = "")]
    fn leading_zero_fails() {
        let (_bytes, _num): (&[u8], i32) = integer(b"i014e")
            .expect("This should panic because integers with leading zeroes are invalid.");
    }

    #[test]
    #[cfg(bigint)]
    fn parse_bigint_pos() -> Result<(), BencodeError> {
        let big_num = format!(
            "{}",
            u128::MAX.to_bigint().unwrap() + u128::MAX.to_bigint().unwrap()
        );
        let (_bytes, _num_big) = integer::<BigInt>(big_num.as_bytes())?;
        Ok(())
    }

    #[test]
    #[cfg(bigint)]
    fn parse_bigint_neg() -> Result<(), BencodeError> {
        let big_num = format!(
            "{}",
            i128::MIN.to_bigint().unwrap() - u128::MAX.to_bigint().unwrap()
        );
        let (_bytes, _num_big) = integer::<BigInt>(big_num.as_bytes())?;
        Ok(())
    }

    #[test]
    #[cfg(bigint)]
    fn parse_biguint_pos() -> Result<(), BencodeError> {
        
    }
}
