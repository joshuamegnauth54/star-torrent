pub mod hybrid;
pub mod metav1;
pub mod metav2;
mod serde_bool_int;

pub use hybrid::Hybrid;
pub use metav1::MetaV1;
pub use metav2::MetaV2;

use crate::files::FileDisplayInfo;
use serde::{Deserialize, Serialize};

/// Metainfo on files shared by torrents.
///
/// The base structure is defined in [BEP-0003](https://www.bittorrent.org/beps/bep_0003.html).
/// Version 2.0 extensions to BEP-0003 are defined in [BEP-0052](https://www.bittorrent.org/beps/bep_0052.html).
///
/// More torrent versions may be added in the future so [Info] is non-exhaustive.
#[non_exhaustive]
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MetaInfo {
    /// Meta version 1
    MetaV1(MetaV1),
    /// Meta version 2
    MetaV2(MetaV2),
    /// Backwards compatible amalgamate of all versions.
    Hybrid(Hybrid),
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};
    #[test]
    fn info_metav1_only() {}

    #[test]
    fn info_metav2_only() {}

    #[test]
    fn info_hybrid() {}
}
