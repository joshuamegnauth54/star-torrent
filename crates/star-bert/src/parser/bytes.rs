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
/// Bencoded bytes are
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

#[cfg(test)]
mod tests {
    use crate::parser::{bytes, BertErrorTrace};
    use nom::Finish;

    // Success case
    #[test]
    fn bytes_to_str_success() -> Result<(), BertErrorTrace<Vec<u8>>> {
        // Cats are cute.
        let cats = "14:A cicák cukik";
        let bytes_str = cats.as_bytes();
        let (remaining, cat_parsed_bytes) = bytes(bytes_str).finish()?;

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
        let bytes_str = "46;fekete bika pata kopog a patika pepita kövein".as_bytes();
        bytes(bytes_str).unwrap();
    }

    // Zero length bytes array
    #[test]
    fn zero_length_bytes_array() -> Result<(), BertErrorTrace<Vec<u8>>> {
        let bytes_str = b"0:";
        let (remaining, parsed_bytes) = bytes(bytes_str).finish()?;

        // Check that both remaining and parsed_bytes are empty.
        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed_bytes.len(), 0);

        Ok(())
    }

    // Try to take 42 bytes from an bytes array which is only 32 bytes long.
    #[test]
    #[should_panic(expected = "")]
    fn length_too_large() {
        let bytes_str = "42:This is not 42 bytes. LOLOL 🐈".as_bytes();
        assert!(bytes_str.len() < 42);
        bytes(bytes_str).unwrap();
    }

    // Try to take a smaller amount of bytes than the actual length.
    // This won't fail, but the next parser will fail because the rest of the bytes
    // won't be in proper Bencode
    #[test]
    fn length_too_small() -> Result<(), BertErrorTrace<Vec<u8>>> {
        // Cabbaging hm hm.
        let bytes_str =
            "48:Elkelkáposztásíthatatlanságoskodásaitokért kivégezlek titeket".as_bytes();
        assert_ne!(48, bytes_str.len());

        // Parse 48 bytes.
        let (remaining, parsed_bytes) = bytes(bytes_str).finish()?;

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
        let bytes_str = b"4:\x00\x00\x00\x00";
        let (remaining, parsed_bytes) = bytes(bytes_str).finish()?;

        // Check that bytes were parsed correctly
        assert_eq!(remaining.len(), 0);
        let parsed_str = std::str::from_utf8(parsed_bytes)
            .map_err(|e| BertErrorTrace::from_bert_error_kind(parsed_bytes, e.into()))?;
        assert_eq!(parsed_str, "\x00\x00\x00\x00");

        Ok(())
    }
}
