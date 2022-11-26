//! Types for cryptography used in torrents.
pub mod md5;
// pub mod rsa;
pub mod sha1;
pub mod sha256;
pub mod signature;

pub mod sha {
    pub use super::sha1::Sha1;
    pub use super::sha256::Sha256;
}
