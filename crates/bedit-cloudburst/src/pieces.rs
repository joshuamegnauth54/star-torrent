use serde::{
    de::{Error as DeError, Unexpected},
    Deserialize, Deserializer, Serialize,
};
use serde_bytes::ByteBuf;
use std::num::NonZeroU64;

use crate::crypto::Sha1Hash;

/// Number of bytes per piece.
///
/// According to the spec, piece length should be greater than 16 KiB and is always a power of two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct PieceLength(NonZeroU64);

impl<'de> Deserialize<'de> for PieceLength {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let piece_length = NonZeroU64::deserialize(deserializer)?;

        if piece_length.get() >= 16 && piece_length.is_power_of_two() {
            Ok(PieceLength(piece_length))
        } else {
            Err(DeError::invalid_value(
                Unexpected::Unsigned(piece_length.into()),
                &"piece length should be greater than 16 and a power of two",
            ))
        }
    }
}

/// Per file SHA-1 hashes.
///
/// The BitTorrent spec specifies `pieces` as a single byte buffer that is a multiple of 20.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Pieces(ByteBuf);

impl<'de> Deserialize<'de> for Pieces {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let pieces = ByteBuf::deserialize(deserializer)?;

        // `pieces` must be a multiple of 20.
        let len = pieces.len();
        if len % 20 == 0 {
            Ok(Pieces(pieces))
        } else {
            Err(DeError::invalid_length(
                len,
                &"length of `pieces` should be a multiple of 20",
            ))
        }
    }
}

impl Pieces {
    /// Iterator over chunks of 20 bytes.
    #[inline]
    pub fn iter_pieces_bytes(&self) -> impl Iterator<Item = &[u8]> + '_ {
        self.0.chunks_exact(20)
    }

    /// Iterator over bytes wrapped in [Sha1Hash].
    #[inline]
    pub fn iter_sha1(&self) -> impl Iterator + '_ {
        self.iter_pieces_bytes().map(|chunk| {
            let bytes: [u8; 20] = chunk.try_into().expect("`Pieces` should always be a multiple of 20 bytes AND chunks_exact() should return a 20 byte chunk.");
            Sha1Hash::from(bytes)
        })
    }
}
