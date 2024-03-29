//! `star-cloudburst` provides strongly typed data structures for serializing and deserializing torrents.
#![feature(let_chains, once_cell_try)]

pub mod crypto;
pub mod files;
pub mod hexadecimal;
pub mod metainfo;
pub mod pieces;
pub mod torrent;
pub mod uri;

pub use pieces::{PieceLength, Pieces};
pub use torrent::Torrent;
