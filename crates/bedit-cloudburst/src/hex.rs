use serde::de::{value::Error as DeError, Error as DeErrorTrait, Unexpected};
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    iter::FusedIterator,
};

const HEX_LOWER: &str = "0123456789abcdef";
// const HEX_UPPER: &str = "0123456789ABCDEF";

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

        if maybe_hex.chars().validate_hex() {
            let bytes = maybe_hex
                .chars()
                .map(|mut ch| {
                    ch.make_ascii_lowercase();

                    // Map character to 0-15
                    HEX_LOWER
                        .find(ch)
                        .ok_or_else(|| {
                            DeError::invalid_value(
                                Unexpected::Char(ch),
                                &"valid hexadecimal characters",
                            )
                        })
                        .map(|position| position.try_into().unwrap_or_else(|_| panic!("Index of character was greater than 255 (can't happen).\nIndex: {position}\n")))
                })
                .collect::<Result<Vec<u8>, _>>()?.into();

            Ok(HexBytes { bytes })
        } else {
            Err(DeError::invalid_value(
                Unexpected::Str(maybe_hex),
                &"valid hexadecimal characters [0-9, a-f, A-F]",
            ))
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
            write!(f, "{:x}", byte)?;
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
        Some((bytes[0] << 4) | bytes[1])
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

    /*
    fn chars_to_packed<I>(self) -> PackedHex<I>
    where
        Self: Sized + Iterator<Item = char>,
        I: Iterator
    {
        PackedHex { iter: self.map(|ch| ch).into_iter() }
    }
    */

    #[inline]
    fn validate_hex(&mut self) -> bool
    where
        Self: Sized + Iterator<Item = char>,
    {
        self.all(|ch| ch.is_ascii_hexdigit())
    }
}

impl<I> Hexadecimal for I where I: Iterator {}
