pub struct Hex;

impl Hex {
    /// Validate byte slice as valid hexadecimal (case insensitive).
    ///
    /// # Examples
    /// ```
    /// use bedit_cloudburst::Hex;
    ///
    /// let james_hoffman = "cafeD00d";
    /// assert!(Hex::validate(james_hoffman.as_bytes()));
    /// ```
    ///
    /// Oops.
    /// ```
    /// use bedit_cloudburst::Hex;
    ///
    /// let giraffe = "*giraffe noises* 🦒";
    /// assert!(!Hex::validate(giraffe.as_bytes()));
    ///
    /// ```
    #[inline]
    pub fn validate<I>(mut bytes: I) -> bool
    where
        I: Iterator<Item = u8>,
    {
        bytes.all(|byte| byte.is_ascii_hexdigit())
    }

    /// Iterate over hexadecimal bytes as nibbles.
    #[inline]
    pub fn as_nibbles<I>(bytes: I) -> impl Iterator<Item = u8>
    where
        I: Iterator<Item = u8>,
    {
        bytes.map(|byte| [byte >> 4, byte & 0b11110000u8]).flatten()
    }
}

/// Yields nibbles from bytes.
pub struct Nibbles<I: Iterator>
where
    I: Iterator<Item = u8>,
{
    iter: I,
}

impl<I> Iterator for Nibbles<I>
where
    I: Iterator<Item = u8>,
{
    type Item = [u8; 2];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.iter.next()?;
        Some([byte >> 4, byte & 0b00001111u8])
    }
}

pub trait Hexadecimal {
    fn nibbles(self) -> Nibbles<Self>
    where
        Self: Sized + Iterator<Item = u8>;

    fn validate_hex(&mut self) -> bool;
}

impl<I> Hexadecimal for I
where
    I: Iterator<Item = u8>,
{
    #[inline]
    fn nibbles(self) -> Nibbles<Self>
    where
        Self: Sized + Iterator,
    {
        Nibbles { iter: self }
    }

    #[inline]
    fn validate_hex(&mut self) -> bool
    where
        Self: Sized + Iterator,
    {
        self.all(|byte| byte.is_ascii_hexdigit())
    }
}
