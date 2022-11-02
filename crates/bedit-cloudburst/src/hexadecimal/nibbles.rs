use std::iter::FusedIterator;

// Pack two hex bytes into a single byte.
#[inline]
pub(super) fn pack_bytes(bytes: [u8; 2]) -> u8 {
    (bytes[0] << 4) | bytes[1]
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
        Some(pack_bytes(bytes))
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
