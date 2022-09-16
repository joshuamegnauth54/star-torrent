//! Type safe torrent file attributes.
//!
//! [BEP-0047](https://www.bittorrent.org/beps/bep_0047.html) defines extra metadata for torrent files.
//! One of these additions is `attr`, a variable length string that lists the attributes of a file.
//! [FileAttribute] wraps the individual attributes while [TorrentFileAttributes] wraps the string.
//! Both of these types verify the input as well as provide serialization.

// Using ArrayVec: https://nnethercote.github.io/perf-book/heap-allocations.html
use arrayvec::ArrayVec;
use itertools::Itertools;
use serde::{
    de::{value::Error as DeError, Error as DeErrorTrait},
    Deserialize, Serialize,
};
use std::fmt::{self, Display, Formatter};

// Valid, lower cased file attributes.
const FILE_ATTRIBUTE_EXPECTED: [&str; 4] = ["x", "h", "p", "l"];

/// File attributes.
///
/// Executable = 'x'
///
/// Hidden = 'h'
///
/// Padding = 'p'
///
/// Symlink = 'l'
///
/// Extended file properties are defined in [BEP-0047](https://www.bittorrent.org/beps/bep_0047.html).
/// Counter to the spec, conversions from [char] and [str] slices are currently fallible. However this may change in the future.
#[derive(Clone, Copy, Debug)]
pub enum FileAttribute {
    Executable,
    Hidden,
    Padding,
    Symlink,
}

impl TryFrom<char> for FileAttribute {
    type Error = DeError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase() {
            'x' => Ok(Self::Executable),
            'h' => Ok(Self::Hidden),
            'p' => Ok(Self::Padding),
            'l' => Ok(Self::Symlink),
            _ => Err(DeError::unknown_variant(
                &value.to_string(),
                &FILE_ATTRIBUTE_EXPECTED,
            )),
        }
    }
}

impl TryFrom<&str> for FileAttribute {
    type Error = DeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Not using to_lowercase() because it returns a String.
        match value {
            "x" | "X" => Ok(Self::Executable),
            "h" | "H" => Ok(Self::Hidden),
            "p" | "P" => Ok(Self::Padding),
            "l" | "L" => Ok(Self::Symlink),
            _ => Err(DeError::unknown_variant(value, &FILE_ATTRIBUTE_EXPECTED)),
        }
    }
}

impl From<FileAttribute> for &str {
    fn from(other: FileAttribute) -> Self {
        match other {
            FileAttribute::Executable => "x",
            FileAttribute::Hidden => "h",
            FileAttribute::Padding => "p",
            FileAttribute::Symlink => "l",
        }
    }
}

impl Serialize for FileAttribute {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str((*self).into())
    }
}

impl<'de> Deserialize<'de> for FileAttribute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let maybe_attr = char::deserialize(deserializer)?;
        maybe_attr
            .try_into()
            // To do: Can't constraint the type of D::Error to DeError yet.
            .map_err(|_| {
                D::Error::unknown_variant(&maybe_attr.to_string(), &FILE_ATTRIBUTE_EXPECTED)
            })
    }
}

// ********************************
// TorrentFileAttributes and impls.
// ********************************

/// Multiple [FileAttribute]`s wrapped for serialization and deserialization.
///
/// The `attr` field is stored as a bencoded string as per [BEP-047](https://www.bittorrent.org/beps/bep_0047.html).
/// [TorrentFileAttributes] wraps an implemention defined vector (currently an [ArrayVec]) of [FileAttribute]s that deserializes
/// to and serializes from a [String].
///
/// # Examples
/// Deserialize to a strongly typed `struct` and back to a [String].
/// ```
/// use bedit_torrent::TorrentFileAttributes;
/// use serde::{Deserialize, Serialize};
/// use serde_bencode::Error;
///
/// let attrs = "2:lx";
/// let torrent_attrs: TorrentFileAttributes = serde_bencode::from_str(attrs)?;
/// let attrs_se = serde_bencode::to_string(&torrent_attrs)?;
/// assert_eq!(attrs, attrs_se);
/// # Ok::<(), Error>(())
/// ```
///
/// Deserialization drops duplicates and sorts the result.
/// ```
/// use bedit_torrent::TorrentFileAttributes;
/// use serde::Deserialize;
/// use serde_bencode::Error;
///
/// let attrs = "23:plxhhxXpPxlLxpphXXXhlLL";
/// let torrent_attrs: TorrentFileAttributes = serde_bencode::from_str(attrs)?;
/// assert_eq!("hlpx", torrent_attrs.to_string());
/// # Ok::<(), Error>(())
/// ```
///
/// [TryFrom] is implemented for [TorrentFileAttributes].
/// ```
/// use bedit_torrent::TorrentFileAttributes;
/// use serde_bencode::Error;
///
/// let attrs = "hlpx";
/// let torrent_attrs: TorrentFileAttributes = attrs.try_into()?;
/// assert_eq!("hlpx", torrent_attrs.to_string());
/// # Ok::<(), Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct TorrentFileAttributes(ArrayVec<FileAttribute, 4>);

impl Display for TorrentFileAttributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let attrs: String = self
            .0
            .iter()
            .cloned()
            .map(<FileAttribute as Into<&str>>::into)
            .collect();
        write!(f, "{}", attrs)
    }
}

impl TryFrom<&str> for TorrentFileAttributes {
    type Error = DeError;

    // Convert a &str containing any case insensitive combination of 'x', 'h', 'p', 'l'
    // to a vector of [FileAttribute].
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let attrs_parsed = value
            .chars()
            .map(|ch| ch.to_ascii_lowercase())
            // Sort so that I could potentially intern the Strings produced during deserialization in the future.
            .sorted()
            // Dedup for the same reason as sorting - plus there is no reason for dupes here.
            .dedup()
            .map(|maybe_attr| maybe_attr.try_into())
            .collect::<Result<ArrayVec<_, 4>, _>>()?;

        Ok(TorrentFileAttributes(attrs_parsed))
    }
}

impl<'de> Deserialize<'de> for TorrentFileAttributes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: Figure out how to deserialize as a borrowed str.
        let attr = String::deserialize(deserializer)?;
        attr.as_str()
            .try_into()
            .map_err(|_| D::Error::unknown_variant(&attr, &FILE_ATTRIBUTE_EXPECTED))
    }
}

impl Serialize for TorrentFileAttributes {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
