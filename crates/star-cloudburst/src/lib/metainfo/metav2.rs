use crate::{
    crypto::sha1::Sha1,
    files::{FileDisplayInfo, FileTree},
    metainfo::serde_bool_int::{bool_from_int, bool_to_int},
    pieces::PieceLength,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::num::NonZeroU8;

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MetaV2 {
    #[serde(rename = "file tree")]
    pub file_tree: FileTree,
    pub name: String,
    pub meta_version: NonZeroU8,
    #[serde(rename = "piece length")]
    pub piece_length: PieceLength,
    #[serde(
        default,
        deserialize_with = "bool_from_int",
        serialize_with = "bool_to_int"
    )]
    pub private: bool,
    #[serde(rename = "root hash")]
    pub root_hash: Sha1,
}
