use std::num::NonZeroU64;

use serde_bencode::Error as BencodeError;
use thiserror::Error;

/// Torrent serializing and deserializing errors.
#[derive(Debug, Error)]
pub enum ParseTorrentError {
    #[error("Ambiguous info dict (w.r.t. files): {0}")]
    AmbiguousFiles(&'static str),
    #[error("Bencode serializing/deserializing")]
    Bencode(#[from] BencodeError),
    #[error("Invalid torrent metainfo version: {0}")]
    InvalidVersion(u8),
    #[error("Piece length too small: {0}")]
    PieceLength(NonZeroU64),
}
