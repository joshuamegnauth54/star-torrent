//! Recursive descent Bencode parser.

use super::parser_error::{BertErrorKind, BertErrorTrace};
use nom::{
    branch::{alt, permutation},
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{cond, map_res, not, recognize},
    error::context,
    sequence::{delimited, pair},
    IResult,
};
use num_integer::Integer;
use std::{fmt::Debug, str::FromStr};

/// Parse a Bencoded integer from bytes.
///
/// Integers are always base ten numbers delimited with 'i' and 'e'. For example, the number 14 Bencoded is `i14e`.
/// Integers are signed; `i-28e` and `i42e` are both valid.
/// Integers cannot start with a leading 0. `i042e` is invalid as is `i-0e` `i0e` is valid of course.
/// Per the spec: "Integers have no size limitation"
/// # Examples
///
/// A basic, positive integer.
///
/// ```rust
/// use star_bert::parser::integer;
/// use star_bert::parser::BertErrorTrace;
///
/// let (_bytes, num) = integer::<u8>(b"i14e")?;
/// assert_eq!(num, 14);
/// # Ok::<(), BertErrorTrace>(())
/// ```
///
/// Negative integer
/// ```
/// use star_bert::parser::integer;
/// use star_bert::parser::BertErrorTrace;
///
/// let (_bytes, num) = integer::<i32>(b"i-28e")?;
/// assert_eq!(num, -28);
/// # Ok::<(), BertErrorTrace>(())
/// ```
///
/// ## Arbitrarily sized integers (BigInt)
/// The Bencode spec doesn't define the size of integers. In other words, an int isn't a `i64` or any other type.
/// BigInts can be enabled via the `bigint` feature which is disabled by default.
///
/// ```
/// use star_bert::parser::integer;
/// use star_bert::parser::BertErrorTrace;
/// use num_bigint::BigInt;
///
/// let big_num = format!("i{}e", u128::MAX.to_bigint().unwrap() + 1);
/// let mun_gib = integer::<BigInt>(big_num.as_bytes())?;
/// # Ok::<(), BertErrorTrace>(())
/// ```
// #[inline]
pub fn integer<N>(input: &[u8]) -> IResult<&[u8], N, BertErrorTrace<&[u8]>>
where
    N: Integer + FromStr,
    <N as FromStr>::Err: Debug + Into<BertErrorKind>,
{
    context(
        "arbitrary precision integer",
        map_res(
            delimited(
                // Opening delimiter
                tag("i"),
                // Only parse the digits if the input is not -0\d{0,} or 0\d{1,}
                cond(
                    permutation::<_, _, BertErrorTrace<&[u8]>, _>((
                        // -0 is invalid. It doesn't matter what follows -0 as long as -0 matches.
                        // -0 is invalid thus if the input is only -0 then the parser should reject it.
                        // -01428 is invalid because of the leading 0 so the parser should reject the input as well.
                        not(tag("-0")),
                        // This case handles a preceding 0. I call digit1 because digit0 would pass for `i0e` which is incorrect.
                        not(pair(char('0'), digit1)),
                    ))(input)
                    .is_ok(),
                    // If the condition holds, match either a positive integer (digit1) or a negative
                    // `recognize` returns the consumed input as the result rather than tuples of `pair`
                    alt((digit1, recognize(pair(char('-'), digit1)))),
                ),
                // Closing delimiter
                tag("e"),
            ),
            // Map the result to N, the integer output
            |maybe_num| {
                maybe_num
                    .ok_or(BertErrorKind::Context(
                        "-0 or leading zeroes is invalid Bencode",
                    ))
                    .and_then(bytes_to_str_to_int)
            },
        ),
    )(input)
}

// Helper functions
/// Parse a [u8] slice to [str] and then to impl [Integer].
/// Returns [BertErrorKind] so that [nom::combinator::map_res] may call [nom::error::FromExternalError] to convert the type into [BertErrorTrace].
#[inline]
fn bytes_to_str_to_int<N>(bytes: &[u8]) -> Result<N, BertErrorKind>
where
    N: Integer + FromStr,
    <N as FromStr>::Err: Debug + Into<BertErrorKind>,
{
    std::str::from_utf8(bytes)
        .map_err(Into::into)
        .and_then(|s| s.parse().map_err(Into::into))
}

#[cfg(test)]
mod tests {
    use crate::parser::{
        integer::{bytes_to_str_to_int, integer},
        parser_error::{BertErrorKind, BertErrorTrace},
    };
    use nom::Finish;

    #[cfg(feature = "bigint")]
    use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};

    // Test [bytes_to_str_int] directly
    #[test]
    fn bytes_to_pos_int() -> Result<(), BertErrorKind> {
        let num_raw = b"54";
        let num: i32 = bytes_to_str_to_int(&num_raw[..])?;
        assert_eq!(num, 54);
        Ok(())
    }

    // Leading zeroes are invalid according to the spec
    #[test]
    #[should_panic(expected = "")]
    fn leading_zero_fails() {
        let (_bytes, _num): (&[u8], i32) = integer(b"i014e")
            .expect("This should panic because integers with leading zeroes are invalid.");
    }

    #[test]
    #[should_panic(expected = "")]
    fn missing_delimiters() {
        integer::<u32>(b"14").unwrap();
    }

    // -0 is invalid Bencode.
    #[test]
    #[should_panic(expected = "")]
    fn negative_zero() {
        let bad_zero = b"i-0e";
        let (_bytes, _num) =
            integer::<i32>(&bad_zero[..]).expect("Negative zero is invalid Bencode.");
    }

    // Positive BigInt
    #[test]
    #[cfg(feature = "bigint")]
    fn parse_bigint_pos() -> Result<(), BertErrorTrace<Vec<u8>>> {
        let big_num_raw = u128::MAX.to_bigint().unwrap() * 2;
        let big_num = format!("i{big_num_raw}e",);
        let (_bytes, num_big) = integer::<BigInt>(big_num.as_bytes()).finish()?;

        assert_eq!(big_num_raw, num_big);
        Ok(())
    }

    // Negative BigInt
    #[test]
    #[cfg(feature = "bigint")]
    fn parse_bigint_neg() -> Result<(), BertErrorTrace<Vec<u8>>> {
        let big_num_raw = i128::MIN.to_bigint().unwrap() - u128::MAX.to_bigint().unwrap();
        let big_num = format!("i{big_num_raw}e",);
        let (_bytes, num_big) = integer::<BigInt>(big_num.as_bytes()).finish()?;

        assert_eq!(big_num_raw, num_big);
        Ok(())
    }

    // BigUint
    #[test]
    #[cfg(feature = "bigint")]
    fn parse_biguint_pos() -> Result<(), BertErrorTrace<Vec<u8>>> {
        let big_num_raw = u128::MAX.to_biguint().unwrap() * 2u32;
        let big_num = format!("i{big_num_raw}e",);
        let (_bytes, num_big) = integer::<BigUint>(big_num.as_bytes()).finish()?;

        assert_eq!(big_num_raw, num_big);
        Ok(())
    }
}
