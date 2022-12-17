use crate::{crypto::sha1::Sha1, hexadecimal::HexBytes};
use log::{error, trace};
use serde::{
    de::{Error as DeErrorTrait, Unexpected},
    Deserialize, Deserializer, Serialize,
};
use std::{fmt::Debug, num::NonZeroU64};

const PIECES_DE_TARGET: &str = "star_cloudburst::Piece::deserialize";
const PIECELENGTH_DE_TARGET: &str = "star_cloudburst::PieceLength::deserialize";

/// Number of bytes per piece.
///
/// According to the spec, piece length should be greater than 16 KiB and is always a power of two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct PieceLength(NonZeroU64);

impl<'de> Deserialize<'de> for PieceLength {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        trace!(target: PIECELENGTH_DE_TARGET, "Deserializing PieceLength.");

        let piece_length = NonZeroU64::deserialize(deserializer)?;

        if piece_length.get() >= 16 && piece_length.is_power_of_two() {
            Ok(PieceLength(piece_length))
        } else {
            error!(
                target: PIECELENGTH_DE_TARGET,
                "Invalid piece length: {piece_length}."
            );
            Err(DeErrorTrait::invalid_value(
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
#[serde(transparent)]
pub struct Pieces(HexBytes);

impl<'de> Deserialize<'de> for Pieces {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        trace!(target: PIECES_DE_TARGET, "Deserializing Pieces.");

        // This is already a byte string so I can't really validate it.
        let pieces: HexBytes = match serde_bytes::ByteBuf::deserialize(deserializer) {
            Ok(bytes) => HexBytes::new(bytes),
            Err(e) => {
                error!(
                    target: PIECES_DE_TARGET,
                    "Error deserializing the byte string for `Pieces`.\nError: {e}"
                );
                return Err(e);
            }
        };

        // `pieces` must be a multiple of 20 because they're SHA-1 hashes.
        let len = pieces.len();
        if len % 20 == 0 {
            Ok(Pieces(pieces))
        } else {
            error!(
                target: PIECES_DE_TARGET,
                "Pieces should be a multiple of twenty; got: {len}"
            );
            Err(DeErrorTrait::invalid_length(
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
        self.0.as_slice().chunks(20)
    }

    /// Iterator over bytes wrapped in [Sha1Hash].
    #[inline]
    pub fn iter_sha1(&self) -> impl Iterator + '_ {
        self.iter_pieces_bytes().map(|chunk| {
            let bytes: [u8; 20] = chunk.try_into().expect("`Pieces` should always be a multiple of 20 bytes AND chunks_exact() should return a 20 byte chunk.");
            Sha1::from(bytes)
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len() / 20
    }

    /// Placate Clippy.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
