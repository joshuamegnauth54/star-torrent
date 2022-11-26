use super::nibbles::pack_bytes;
use log::error;
use serde::{
    de::{value::Error as DeError, Error as DeErrorTrait, Unexpected},
    Deserialize, Serialize,
};
use serde_bytes::ByteBuf;
use std::fmt::{self, Binary, Debug, Display, Formatter, LowerHex, UpperHex};

const HEX_LOWER: &str = "0123456789abcdef";
// const HEX_UPPER: &str = "0123456789ABCDEF";
const HEX_EXPECTED: &str = "valid hexadecimal characters [0-9, a-f, A-F]";
const FROMHEXSTR_TARGET: &str = "star_cloudburst::hexadecimal::HexBytes::from_hex_str";

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct HexBytes {
    bytes: ByteBuf,
}

impl HexBytes {
    /// Validate byte slice as valid hexadecimal (case insensitive).
    ///
    /// [Display], [Binary], [UpperHex], and [LowerHex] are implemented for [HexBytes].
    ///
    /// # Examples
    /// ```
    /// use star_cloudburst::hexadecimal::HexBytes;
    /// use serde::de::value::Error;
    ///
    /// let james_hoffman = "cafeD00d";
    /// let coffee_hex = HexBytes::from_hex_str(james_hoffman)?;
    ///
    /// let coffee_str = coffee_hex.to_string();
    /// assert_eq!(coffee_str, "cafed00d");
    ///
    /// let cafe_upper = format!("{:02X}", coffee_hex);
    /// assert_eq!(cafe_upper, "CAFED00D");
    ///
    /// let cafe_lower = format!("{:02x}", coffee_hex);
    /// assert_eq!(cafe_lower, "cafed00d");
    ///
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// Oops.
    /// ```
    /// use star_cloudburst::hexadecimal::HexBytes;
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
        let maybe_hex_len = maybe_hex.len();

        if maybe_hex_len % 2 != 0 || maybe_hex_len == 0 {
            error!(
                target: FROMHEXSTR_TARGET,
                "Invalid hex string length: {maybe_hex_len}"
            );

            Err(DeError::invalid_length(
                maybe_hex_len,
                &"valid hex string lengths are divisible by two and greater than zero",
            ))
        } else {
            let bytes = maybe_hex
                .as_bytes()
                .chunks(2)
                .map(|chunk| {
                    // This won't panic because maybe_hex.len() is divisible by two.
                    // Normally chunks would return the remainder and thus the below would panic.
                    let upper = chunk[0].to_ascii_lowercase() as char;
                    let lower = chunk[1].to_ascii_lowercase() as char;

                    // Uh, I really want to use PackedHex here but hex_to_byte is fallible.
                    match (hex_to_byte(upper), hex_to_byte(lower)) {
                        (Ok(upper), Ok(lower)) => Ok(pack_bytes([upper, lower])),
                        (Err(e), _) | (_, Err(e)) => {
                            error!(target: FROMHEXSTR_TARGET, "ASCII character is out of the range for hex; {upper} {lower}\nError: {e}");
                            Err(e)
                        },
                    }
                })
                .collect::<Result<Vec<u8>, _>>()?
                .into();

            Ok(bytes)
        }
    }

    #[inline]
    pub(crate) fn new(bytes: ByteBuf) -> Self {
        // For some reason, probably due to my own idiocy, I can't use Into to construct HexBytes.
        Self { bytes }
    }

    /// Returns the amount of bytes stored.
    ///
    /// ```rust
    /// use star_cloudburst::hexadecimal::HexBytes;
    /// use serde::de::value::Error;
    ///
    /// let bytes = HexBytes::from_hex_str("dead")?;
    /// assert_eq!(bytes.len(), 2);
    ///
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    /// Iterator over packed hexadecimal bytes.
    #[inline]
    pub fn iter(&self) -> impl Iterator + '_ {
        self.bytes.iter()
    }

    // Proxy function to make implementing traits from [std::fmt] easier.
    fn hex_display_proxy(&self, f: &mut Formatter<'_>) -> Result<(usize, usize), fmt::Error> {
        if f.alternate() {
            write!(f, "0x")?;
        }

        let width = f.width().unwrap_or(0);
        let precision = f.precision().unwrap_or(2);

        Ok((width, precision))
    }
}

impl<B> From<B> for HexBytes
where
    B: Into<Vec<u8>>,
{
    #[inline]
    fn from(bytes: B) -> Self {
        // No validation because these are just bytes.
        Self {
            bytes: ByteBuf::from(bytes.into()),
        }
    }
}

// Bytes are assumed to be packed hexadecimal which is fine because I check it anyway.
impl Display for HexBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for &byte in self.bytes.iter() {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl Binary for HexBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for byte in self.bytes.iter() {
            Binary::fmt(byte, f)?;
            write!(f, " ")?;
        }

        Ok(())
    }
}

impl LowerHex for HexBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (width, precision) = self.hex_display_proxy(f)?;

        for byte in self.bytes.iter() {
            write!(f, "{:0width$.precision$x}", byte)?;
        }

        Ok(())
    }
}

impl UpperHex for HexBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (width, precision) = self.hex_display_proxy(f)?;

        for &byte in self.bytes.iter() {
            write!(f, "{:0width$.precision$X}", byte)?;
        }

        Ok(())
    }
}
