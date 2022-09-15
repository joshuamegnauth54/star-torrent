use super::files::{FileTree, SharedFiles};
use serde::{
    de::{Error as DeError, Unexpected},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_bencode::value::Value;
use serde_bytes::ByteBuf;
use serde_with::skip_serializing_none;
use std::num::{NonZeroU64, NonZeroU8};

/// Metainfo on files shared by torrents.
///
/// The base structure is defined in [BEP-0003](https://www.bittorrent.org/beps/bep_0003.html).
/// Extensions to BEP-0003 are defined in [BEP-0052](https://www.bittorrent.org/beps/bep_0052.html).
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Info {
    MetaV1(MetaV1),
    MetaV2(MetaV2),
    Hybrid(Hybrid),
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MetaV1 {
    #[serde(default)]
    pub files: Option<Vec<SharedFiles>>,
    #[serde(default)]
    pub length: Option<u64>,
    #[serde(default)]
    pub md5sum: Option<String>,
    pub name: String,
    pub pieces: ByteBuf,
    #[serde(rename = "piece length")]
    pub piece_length: NonZeroU64,
    #[serde(
        default,
        deserialize_with = "bool_from_int",
        serialize_with = "bool_to_int"
    )]
    pub private: Option<bool>,
    #[serde(default, rename = "root hash")]
    pub root_hash: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MetaV2 {
    #[serde(rename = "file tree")]
    pub file_tree: FileTree,
    pub meta_version: NonZeroU8,
    #[serde(rename = "piece length")]
    pub piece_length: NonZeroU64,
}

/// Metainfo on file(s) shared by hybrid torrents.
///
/// Hybrid torrents contain the info dicts for all torrent meta versions.
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Hybrid {
    /// Files shared by version 1 or hybrid torrents.
    #[serde(default)]
    pub files: Option<Vec<SharedFiles>>,
    /// Version 2 or hybrid styled file dictionaries.
    #[serde(default, rename = "file tree")]
    pub file_tree: Option<FileTree>,
    /// Length of the file in bytes.
    /// This is present if the torrent only shares one file.
    #[serde(default)]
    pub length: Option<u64>,
    /// Torrent file meta version
    ///
    /// This is specified in BEP-0052 which revises the original torrent format.
    /// Meta version must be greater than or equal to 2. Meta version is increased for
    /// major changes such as deprecating a hash algorithm in favor of a new algo.
    #[serde(default, rename = "meta version")]
    pub meta_version: Option<u8>,
    /// Shared file's MD5 hash.
    #[serde(default)]
    pub md5sum: Option<String>,
    /// Suggested name of the file or subdirectory to which to save multiple files.
    ///
    /// `name` is a suggestion - files or directories don't have to rigidly follow it.
    pub name: String,
    /// A SHA-1 hash list of each piece concatenated into a string.
    /// The resulting string's length is a multiple of 20 bytes. The position of each hash
    /// corresponds to a file in `files`.
    pub pieces: Option<ByteBuf>,
    /// Number of bytes per piece.
    ///
    /// BEP-0003 states that the length is almost always a power of two and usually 2^18.
    #[serde(rename = "piece length")]
    pub piece_length: NonZeroU64,
    /// Torrent is restricted to private trackers.
    ///
    /// [BEP-0027](https://www.bittorrent.org/beps/bep_0027.html)
    /// Private torrents are only advertised on a private tracker. The swarm is limited to
    /// that particular tracker even if multiple trackers are specified. Torrent clients should
    /// disconnect from all peers if trackers are switched
    #[serde(
        default,
        deserialize_with = "bool_from_int",
        serialize_with = "bool_to_int"
    )]
    pub private: Option<bool>,
    /// Merkle tree root hash.
    ///
    /// [BEP-0030](https://www.bittorrent.org/beps/bep_0030.html) adds Merkle trees to reduce torrent file
    /// sizes. Instead of a hash per piece, a Merkle torrent contains the root hash of the tree through which
    /// the hashes of the subseqeuent pieces may be derived.
    #[serde(default, rename = "root hash")]
    pub root_hash: Option<String>,
}

/// Deserialize Option<u8> to Option<bool>.
fn bool_from_int<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Int(i) => match i {
            0 => Ok(Some(false)),
            1 => Ok(Some(true)),
            nonbool => Err(DeError::invalid_value(
                Unexpected::Unsigned(nonbool as u64),
                &"zero or one",
            )),
        },
        Value::Bytes(maybe_none) if maybe_none.is_empty() => Ok(None),
        wrong => {
            let unexpected = match wrong {
                Value::List(_) => Unexpected::Seq,
                Value::Dict(_) => Unexpected::Map,
                Value::Bytes(bytes) if !bytes.is_empty() => {
                    Unexpected::Other("&[u8] that's not empty")
                }
                _ => unreachable!("Value::Int and Value::Bytes([]) were checked earlier."),
            };

            Err(DeError::invalid_type(unexpected, &"zero or one"))
        }
    }
}

/// Serialize Option<bool> to Option<u8>.
fn bool_to_int<S>(private: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match private {
        Some(true) => serializer.serialize_some(&1),
        Some(false) => serializer.serialize_some(&0),
        None => serializer.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::{bool_from_int, bool_to_int};
    use serde::de::{
        value::{Error as DeError, StrDeserializer},
        Error as SerdeError, IntoDeserializer,
    };
    use std::error::Error;

    #[test]
    fn bool_from_int_valid() -> Result<(), Box<dyn Error>> {
        let states = [("i0e", false), ("i1e", true)];

        for (value, expected) in states {
            let mut deserializer = serde_bencode::Deserializer::new(value.as_bytes());
            let maybe_bool = bool_from_int(&mut deserializer)?
                .ok_or_else(|| DeError::custom("Expected Some({expected})"))?;

            if maybe_bool != expected {
                Err(DeError::custom("Expected {expected}"))?
            }
        }
        Ok(())
    }

    #[test]
    #[should_panic(expected = "Invalid Value: integer `14` (expected: `zero or one`)")]
    fn bool_from_int_invalid() {
        let mut deserializer = serde_bencode::Deserializer::new("i14e".as_bytes());
        bool_from_int(&mut deserializer)
            .expect("Invalid Value: integer `14` (expected: `zero or one`)");
    }

    #[test]
    fn bool_from_int_none() {
        let deserializer: StrDeserializer<'static, DeError> = "".into_deserializer();
        bool_from_int(deserializer).unwrap();
    }

    #[test]
    fn int_from_bool() -> Result<(), serde_bencode::Error> {
        let mut serializer = serde_bencode::Serializer::new();
        bool_to_int(&Some(true), &mut serializer)?;

        let bytes_ser = serializer.into_vec();
        assert!(
            bytes_ser == "i1e".as_bytes(),
            "Some(true) wasn't serialized"
        );

        Ok(())
    }
}