use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::hexadecimal::HexBytes;
// use super::rsa::Rsa;

/// Additional info for `Signature`; unused.
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct SignInfo {}

/// Signatures for signed torrents. [BEP-0035](https://www.bittorrent.org/beps/bep_0035.html)
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct Signature {
    /// X.509 certificate used to sign the torrent. The user should have a certificate elsewhere if this is missing.
    #[serde(default)]
    certificate: Option<String>,
    /// Extension info (currently unspecified)
    #[serde(default)]
    info: Option<SignInfo>,
    /// RSA signature of torrent's `Info` and `Signature`'s `SignInfo` if present.
    signature: HexBytes,
}
