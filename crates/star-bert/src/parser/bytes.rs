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

pub fn to_utf8(input: &[u8]) -> Result<&str, BertErrorTrace<&[u8]>> {

}

#[cfg(test)]
mod tests {
    use crate::parser::{bytes, BertErrorTrace};
    use nom::{error::ErrorKind, error::FromExternalError, Finish};

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
            "24:Elkelkáposztásíthatatlanságoskodásaitokért kivégezlek titeket".as_bytes();
        assert_ne!(24, bytes_str.len());

        // Parse 48 bytes.
        let (_remaining, parsed_bytes) = bytes(bytes_str).finish()?;
        assert_eq!(parsed_bytes.len(), 48);
        let parsed_str = std::str::from_utf8(parsed_bytes)
            .map_err(|e| BertErrorTrace::from_external_error(parsed_bytes, ErrorKind::MapRes, e))?;
        assert_eq!(parsed_str, "Elkelkáposztásíthatatlanságoskodásaitokért");

        Ok(())
    }
}
