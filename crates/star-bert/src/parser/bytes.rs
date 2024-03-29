//! Parse Bencoded bytes buffers or [String]s.
//!
//! [BEP-0003](https://www.bittorrent.org/beps/bep_0003.html)
use crate::parser::{integer::bytes_to_str_to_int, BertErrorTrace};
use nom::{
    bytes::complete::take,
    character::complete::{char, digit1},
    combinator::{flat_map, map_res},
    error::context,
    sequence::terminated,
    IResult,
};

/// Parse a Bencoded bytes buffer.
///
/// Bencoded bytes are prefixed by the length and a colon followed by the bytes
/// themselves. Strings are encoded in the same way, but Bencoded strings must
/// be valid unicode.
///
/// [bytes_str] is a convenience function to parse a bytes array into a valid
/// unicode &[str].
///
/// # Examples
/// ```
/// use nom::Finish;
/// use star_bert::parser::bytes;
/// # use star_bert::parser::BertErrorTrace;
///
/// let dood = [b'4', b':', 0xCA, 0xFE, 0xD0, 0x0D];
/// let (remaining, parsed_dood) = bytes(&dood).finish()?;
/// assert_eq!(&dood[2..], parsed_dood);
/// # assert_eq!(remaining.len(), 0);
/// # Ok::<(), BertErrorTrace<Vec<u8>>>(())
/// ```
pub fn bytes(input: &[u8]) -> IResult<&[u8], &[u8], BertErrorTrace<&[u8]>> {
    context(
        "[Parse] {bytes} Bytes array/string",
        flat_map(
            // Map the result of parsing the length to `bytes_to_str_to_int`
            map_res(
                // Parse length and colon - for example, `14:`
                terminated(
                    context(
                        "[Expected] {bytes} Bytes length as positive integer",
                        digit1,
                    ),
                    // Throw away delimiter
                    context("[Expected] {bytes} Delimiter `:`", char(':')),
                ),
                // Parse length bytes as usize
                bytes_to_str_to_int::<usize>,
            ),
            // Take N bytes
            take,
        ),
    )(input)
}

/// Parse a Bencoded bytes buffer into a valid UTF-8 [str].
///
/// Bencoded strings must be valid UTF-8 according to the [spec](http://bittorrent.org/beps/bep_0003.html).
/// [bytes_str] is a convenience function to parse a Bencoded bytes array
/// followed by parsing the result into a UTF-8 string slice.
///
/// # Examples
///
/// Valid UTF-8 string.
/// ```
/// use nom::Finish;
/// use star_bert::parser::bytes_str;
/// # use star_bert::parser::BertErrorTrace;
///
/// let storm = "18:Il pleut à mourir";
/// let (remaining, storm_parsed) = bytes_str(storm.as_bytes()).finish()?;
/// assert_eq!(&storm[3..], storm_parsed);
/// # assert_eq!(remaining.len(), 0);
/// # Ok::<(), BertErrorTrace<Vec<u8>>>(())
/// ```
///
/// Plain ASCII string
/// ```
/// use nom::Finish;
/// use star_bert::parser::bytes_str;
/// # use star_bert::parser::BertErrorTrace;
///
/// let icon = "47:To win the game, you must kill me, John Romero.";
/// let (remaining, icon_parsed) = bytes_str(icon.as_bytes()).finish()?;
/// assert_eq!(&icon[3..], icon_parsed);
/// # assert_eq!(remaining.len(), 0);
/// # Ok::<(), BertErrorTrace<Vec<u8>>>(())
/// ```
///
/// Invalid UTF-8
/// From [Markus Kuhn's decoder tests](https://www.cl.cam.ac.uk/~mgk25/ucs/examples/UTF-8-test.txt)
/// ```
/// use nom::Finish;
/// use star_bert::parser::bytes_str;
///
/// let bytes_prefix = "6:";
/// let not_utf8 = [0xfc, 0x80, 0x80, 0x80, 0x80, 0xaf];
/// let nope: Vec<u8> = bytes_prefix.bytes().chain(not_utf8.into_iter()).collect();
/// assert!(bytes_str(&nope).is_err());
/// ```
#[inline]
pub fn bytes_str(input: &[u8]) -> IResult<&[u8], &str, BertErrorTrace<&[u8]>> {
    let (remaining, bytes) = bytes(input)?;
    std::str::from_utf8(bytes)
        .map_err(|kind| nom::Err::Failure(BertErrorTrace::from_bert_error_kind(bytes, kind.into())))
        .map(|parsed_str| (remaining, parsed_str))
}

#[cfg(test)]
mod tests {
    use crate::parser::{bytes, bytes_str, BertErrorTrace};
    use nom::Finish;

    // Success case
    #[test]
    fn bytes_to_str_success() -> Result<(), BertErrorTrace<Vec<u8>>> {
        // Cats are cute.
        let cats = "14:A cicák cukik";
        let cats_bytes_str = cats.as_bytes();
        let (remaining, cat_parsed_bytes) = bytes(cats_bytes_str).finish()?;

        // Check if the bytes were parsed properly
        assert_eq!(remaining.len(), 0);
        let cats_parsed = std::str::from_utf8(cat_parsed_bytes)
            .map_err(|e| BertErrorTrace::from_bert_error_kind(cat_parsed_bytes, e.into()))?;
        assert!(cats.chars().skip(3).eq(cats_parsed.chars()));

        Ok(())
    }

    // Incorrect delimiter
    #[test]
    #[should_panic(expected = "")]
    fn missing_colon() {
        let twist_bytes_str = "46;fekete bika pata kopog a patika pepita kövein".as_bytes();
        bytes(twist_bytes_str).unwrap();
    }

    // Zero length bytes array
    #[test]
    fn zero_length_bytes_array() -> Result<(), BertErrorTrace<Vec<u8>>> {
        let zero_bytes_str = b"0:";
        let (remaining, parsed_bytes) = bytes(zero_bytes_str).finish()?;

        // Check that both remaining and parsed_bytes are empty.
        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed_bytes.len(), 0);

        Ok(())
    }

    // Try to take 42 bytes from an bytes array which is only 32 bytes long.
    #[test]
    #[should_panic(expected = "")]
    fn length_too_large() {
        let wrong_bytes_str = "42:This is not 42 bytes. LOLOL 🐈".as_bytes();
        assert!(wrong_bytes_str.len() < 42);
        bytes(wrong_bytes_str).unwrap();
    }

    // Try to take a smaller amount of bytes than the actual length.
    // This won't fail, but the next parser will fail because the rest of the bytes
    // won't be in proper Bencode
    #[test]
    fn length_too_small() -> Result<(), BertErrorTrace<Vec<u8>>> {
        // Cabbaging hm hm.
        let emi_bytes_str =
            "48:Elkelkáposztásíthatatlanságoskodásaitokért kivégezlek titeket".as_bytes();
        assert_ne!(48, emi_bytes_str.len());

        // Parse 48 bytes.
        let (remaining, parsed_bytes) = bytes(emi_bytes_str).finish()?;

        // Check that parsed bytes were parsed properly
        assert_eq!(parsed_bytes.len(), 48);
        let parsed_str = std::str::from_utf8(parsed_bytes)
            .map_err(|e| BertErrorTrace::from_bert_error_kind(parsed_bytes, e.into()))?;
        assert_eq!(parsed_str, "Elkelkáposztásíthatatlanságoskodásaitokért");

        // Check that the remaining bytes array is the correct length
        assert_eq!(remaining.len(), 20);

        Ok(())
    }

    // A bytes array of null characters.
    #[test]
    fn null_bytes() -> Result<(), BertErrorTrace<Vec<u8>>> {
        let unparsed = b"4:\x00\x00\x00\x00";
        let (remaining, parsed_bytes) = bytes(unparsed).finish()?;

        // Check that bytes were parsed correctly
        assert_eq!(remaining.len(), 0);
        let parsed_str = std::str::from_utf8(parsed_bytes)
            .map_err(|e| BertErrorTrace::from_bert_error_kind(parsed_bytes, e.into()))?;
        assert_eq!(parsed_str, "\x00\x00\x00\x00");

        Ok(())
    }

    // Test bytes_str directly
    #[test]
    fn bytes_str_works() -> Result<(), BertErrorTrace<Vec<u8>>> {
        let rin_unparsed = "9:星空凛";
        let (remaining, rin_parsed) = bytes_str(rin_unparsed.as_bytes()).finish()?;

        // Check that bytes were parsed correctly.
        assert_eq!(remaining.len(), 0);
        assert_eq!(&rin_unparsed[2..], rin_parsed);

        Ok(())
    }
}
