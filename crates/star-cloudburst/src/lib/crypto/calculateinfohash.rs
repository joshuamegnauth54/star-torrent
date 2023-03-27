use digest::{Digest, FixedOutput, HashMarker, OutputSizeUser};
use log::debug;
use serde::{ser::Error as SerError, Serialize};

const CALCULATEINFOHASH_TARGET: &str =
    "star_cloudburst::crypto::CalculateInfoHash::calculate_infohash";

pub(crate) trait CalculateInfoHash<const HASHSIZE: usize> {
    // Serialization error.
    type Error: SerError;
    // Info dict.
    type Info: Serialize;
    // Hasher type based on RustCrypto traits.
    type Hasher: Default + HashMarker + FixedOutput + OutputSizeUser;

    // Calculate info hash based on a specific digest, such as SHA2.
    fn calculate_infohash(info: &Self::Info) -> Result<Self, Self::Error>
    where
        Self: Sized + From<[u8; HASHSIZE]>,
    {
        debug!(
            target: CALCULATEINFOHASH_TARGET,
            "Calculating an info hash; expected length {HASHSIZE}."
        );
        // Serialize info dict into a String because info hashes are calculated from
        // Bencoded info dicts.
        let info_se = serde_bencode::to_string(info).map_err(SerError::custom)?;

        // Hash the info dict String into whatever digest is specified (i.e. SHA-2)
        let mut hasher: Self::Hasher = Digest::new();
        hasher.update(info_se.as_bytes());
        let result = hasher.finalize();

        // Convert the final result into HexBytes.
        //let hexbytes: HexBytes = result.to_vec().into();
        let result: [u8; HASHSIZE] = result.as_slice().try_into().map_err(|e| {
            SerError::custom(format!(
                "Byte slice size should be {HASHSIZE} instead of {}.\nTryFrom: {e}",
                result.len()
            ))
        })?;
        Ok(result.into())
    }
}
