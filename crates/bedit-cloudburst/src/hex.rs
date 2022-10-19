use serde::de::{value::Error as DeError, Error as DeErrorTrait, Unexpected};
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    iter::FusedIterator,
};

const HEX_LOWER: &str = "0123456789abcdef";
// const HEX_UPPER: &str = "0123456789ABCDEF";
const HEX_EXPECTED: &str = "valid hexadecimal characters [0-9, a-f, A-F]";

// Map valid hex character to 0-15
fn hex_to_byte(ch: char) -> Result<u8, DeError> {
    HEX_LOWER
        .find(ch)
        .ok_or_else(|| DeError::invalid_value(Unexpected::Char(ch), &HEX_EXPECTED))
        .map(|position| {
            position.try_into().unwrap_or_else(|_| {
                panic!(
                    "Index of character was greater than 255 (can't happen).\nIndex: {position}\n"
                )
            })
        })
}

// Pack two hex bytes into a single byte.
#[inline]
fn pack(bytes: [u8; 2]) -> u8 {
    (bytes[0] << 4) | bytes[1]
}

pub struct HexBytes<'bytes> {
    bytes: Cow<'bytes, [u8]>,
}

impl HexBytes<'_> {
    /// Validate byte slice as valid hexadecimal (case insensitive).
    ///
    /// # Examples
    /// ```
    /// use bedit_cloudburst::HexBytes;
    /// use serde::de::value::Error;
    ///
    /// let james_hoffman = "cafeD00d";
    /// let coffee_hex = HexBytes::from_hex_str(james_hoffman)?;
    ///
    /// let coffee_str = coffee_hex.to_string();
    /// assert_eq!(coffee_str, "cafed00d");
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// Oops.
    /// ```
    /// use bedit_cloudburst::HexBytes;
    ///
    /// let giraffe = "*giraffe noises* 🦒";
    /// let hexraffe = HexBytes::from_hex_str(giraffe);
    ///
    /// assert!(hexraffe.is_err())
    /// ```
    pub fn from_hex_str<S>(maybe_hex: S) -> Result<Self, DeError>
    where
        S: AsRef<str>,
    {
        let maybe_hex = maybe_hex.as_ref();

        if maybe_hex.len() % 2 != 0 {
            Err(DeError::invalid_length(
                maybe_hex.len(),
                &"valid hex string lengths are divisible by two",
            ))
        } else {
            let bytes = maybe_hex
                .as_bytes()
                .chunks(2)
                .map(|chunk| {
                    // This won't panic because maybe_hex.len() is divisible by two.
                    // Normally chunks would return the remainder and thus the below would panic.
                    let upper = chunk[0].to_ascii_lowercase();
                    let lower = chunk[1].to_ascii_lowercase();

                    // Uh, I really want to use PackedHex here but hex_to_byte is fallible.
                    match (hex_to_byte(upper as char), hex_to_byte(lower as char)) {
                        (Ok(upper), Ok(lower)) => Ok(pack([upper, lower])),
                        (Err(e), _) | (_, Err(e)) => Err(e),
                    }
                })
                .collect::<Result<Vec<u8>, _>>()?
                .into();

            Ok(HexBytes { bytes })
        }
    }
}

impl<'bytes, B> From<B> for HexBytes<'bytes>
where
    B: Into<Cow<'bytes, [u8]>>,
{
    fn from(bytes: B) -> Self {
        // No validation because these are just bytes.
        Self {
            bytes: bytes.into(),
        }
    }
}

impl Display for HexBytes<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for &byte in self.bytes.iter() {
            // Bytes are assumed to be packed hexadecimal
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

/// Yields nibbles from bytes.
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Nibbles<I> {
    iter: I,
}

/// Yields packed bytes from nibbles.
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct PackedHex<I> {
    iter: I,
}

// Adapter to unpack a `u8` to `[u8; 2]` (nibbles).
impl<I> Iterator for Nibbles<I>
where
    I: Iterator<Item = u8>,
{
    type Item = [u8; 2];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.iter.next()?;
        Some([byte >> 4, byte & 0x0F])
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I> FusedIterator for Nibbles<I> where I: Iterator<Item = u8> {}
impl<I> ExactSizeIterator for Nibbles<I>
where
    I: Iterator<Item = u8> + ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I> Nibbles<I>
where
    I: Iterator,
{
    #[inline]
    pub fn packed_hex(self) -> PackedHex<Self>
    where
        Self: Sized + Iterator<Item = [u8; 2]>,
    {
        PackedHex { iter: self }
    }
}

// Adapter to pack `[u8; 2]` to `u8`.
impl<I> Iterator for PackedHex<I>
where
    I: Iterator<Item = [u8; 2]>,
{
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.iter.next()?;
        Some(pack(bytes))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I> FusedIterator for PackedHex<I> where I: Iterator<Item = [u8; 2]> {}

impl<I> ExactSizeIterator for PackedHex<I>
where
    I: Iterator<Item = [u8; 2]> + ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

pub trait Hexadecimal {
    #[inline]
    fn nibbles(self) -> Nibbles<Self>
    where
        Self: Sized + Iterator<Item = u8>,
    {
        Nibbles { iter: self }
    }

    #[inline]
    fn packed_hex(self) -> PackedHex<Self>
    where
        Self: Sized + Iterator<Item = [u8; 2]>,
    {
        PackedHex { iter: self }
    }

    #[inline]
    fn validate_hex(&mut self) -> bool
    where
        Self: Sized + Iterator<Item = char>,
    {
        self.all(|ch| ch.is_ascii_hexdigit())
    }
}

impl<I> Hexadecimal for I where I: Iterator {}
