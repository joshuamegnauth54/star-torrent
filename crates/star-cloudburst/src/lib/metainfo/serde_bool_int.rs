use log::{debug, trace};
use serde::{
    de::{Error as DeErrorTrait, Unexpected},
    Deserialize, Deserializer, Serializer,
};

const BOOLFROMINT_DE_TARGET: &str = "star_cloudburst::info::bool_from_int";

/// Deserialize u8 to bool.
pub(super) fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    trace!(
        target: BOOLFROMINT_DE_TARGET,
        "Deserializing `bool` from an integer (likely the `private` field"
    );

    match u8::deserialize(deserializer) {
        Ok(maybe_bool) => match maybe_bool {
            0 => Ok(false),
            1 => Ok(true),
            nonbool => Err(DeErrorTrait::invalid_value(
                Unexpected::Unsigned(nonbool as u64),
                &"zero or one",
            )),
        },
        Err(error) => {
            debug!(target: BOOLFROMINT_DE_TARGET, "Deserializing `private` failed which most likely means the field doesn't exist. Documenting anyways.\nError: {error}");
            Ok(false)
        }
    }
}

/// Serialize bool to u8.
#[inline]
pub(super) fn bool_to_int<S>(private: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u8(*private as u8)
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
            let maybe_bool = bool_from_int(&mut deserializer)?;

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
        bool_to_int(&true, &mut serializer)?;

        let bytes_ser = serializer.into_vec();
        assert!(bytes_ser == "i1e".as_bytes(), "`true` wasn't serialized");

        Ok(())
    }
}
