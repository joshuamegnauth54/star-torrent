//! Types for cryptography used in torrents.
pub mod md5;
// pub mod rsa;
pub(crate) mod calculateinfohash;
pub mod sha1;
pub mod sha2;
pub mod signature;

pub mod sha {
    pub use super::sha1::Sha1;
    pub use super::sha2::Sha2;
}
