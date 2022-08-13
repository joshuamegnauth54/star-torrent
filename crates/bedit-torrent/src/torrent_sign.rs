use serde::{Deserialize, Serialize};

/// Additional info for `Signature`; unused.
#[derive(Debug, Deserialize, Serialize)]
pub struct SignInfo {}

/// Signatures for signed torrents. [BEP-0035](https://www.bittorrent.org/beps/bep_0035.html)
#[derive(Debug, Deserialize, Serialize)]
pub struct Signature {
    /// X.509 certificate used to sign the torrent. The user should have a certificate elsewhere if this is missing.
    #[serde(default)]
    certificate: Option<String>,
    /// Extension info (currently unspecified)
    #[serde(default)]
    info: Option<SignInfo>,
    /// Signature of torrent's `Info` and `Signature`'s `SignInfo` if present.
    signature: String,
}
