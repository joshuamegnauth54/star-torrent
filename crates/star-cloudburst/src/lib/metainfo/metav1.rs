use crate::{
    crypto::md5::Md5,
    files::{FileDisplayInfo, MetaV1FileRepr},
    metainfo::serde_bool_int::{bool_from_int, bool_to_int},
    pieces::{PieceLength, Pieces},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::num::NonZeroU64;



#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MetaV1 {
    pub files: MetaV1FileRepr,
    #[serde(default)]
    pub length: Option<NonZeroU64>,
    #[serde(default)]
    pub md5sum: Option<Md5>,
    pub name: String,
    pub pieces: Pieces,
    #[serde(rename = "piece length")]
    pub piece_length: PieceLength,
    #[serde(
        default,
        deserialize_with = "bool_from_int",
        serialize_with = "bool_to_int"
    )]
    pub private: bool,
}
