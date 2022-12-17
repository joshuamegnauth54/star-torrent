use http::uri::Uri;
use log::trace;
use serde::{
    de::{value::Error as DeError, Error as DeErrorTrait},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    borrow::{Borrow, Cow},
    fmt::{self, Display, Formatter},
    str::FromStr,
};

// Preclude schemes such as `file`
// This should be fairly liberal in what it accepts.
const SANCTIONED_SCHEMES: &[&str] = &[
    "ed2k", "ftp", "http", "https", "gopher", "magnet", "sftp", "tcp", "tftp", "udp", "ws", "wss",
];

// Log targets
const URIWRAPPER_PARSE_TARGET: &str = "star_cloudburst::uri::UriWrapper::from_str";
const URIWRAPPER_DE_TARGET: &str = "star_cloudburst::uri::UriWrapper::deserialize";
const URIWRAPPER_SER_TARGET: &str = "star_cloudburst::uri::UriWrapper::serialize";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UriWrapper(Uri);

impl UriWrapper {
    #[inline]
    pub(crate) fn from_uri_unchecked(uri: Uri) -> Self {
        Self(uri)
    }

    #[inline]
    pub(crate) fn into_inner(self) -> Uri {
        self.0
    }
}

impl<'de> Deserialize<'de> for UriWrapper {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        trace!(target: URIWRAPPER_DE_TARGET, "Deserializing UriWrapper");
        let uri_str: Cow<'de, str> = Cow::deserialize(deserializer)?;
        uri_str.parse().map_err(DeErrorTrait::custom)
    }
}

impl Serialize for UriWrapper {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        trace!(
            target: URIWRAPPER_SER_TARGET,
            "Serializing UriWrapper: {self:?}"
        );
        serializer.serialize_str(&self.0.to_string())
    }
}

impl Display for UriWrapper {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Uri as Display>::fmt(&self.0, f)
    }
}

impl Borrow<Uri> for UriWrapper {
    #[inline]
    fn borrow(&self) -> &Uri {
        &self.0
    }
}

impl FromStr for UriWrapper {
    type Err = DeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        trace!(
            target: URIWRAPPER_PARSE_TARGET,
            "Parsing string `{s}` into `UriWrapper`"
        );

        // Parse into a Uri and then ensure the Uri is correct for torrents.
        s.parse().map_err(DeError::custom).and_then(|uri: Uri| {
            // Disallow relative URIs which can indicate path traversal
            // Absolute URIs work with I.P. addresses too; this is why I can't just use URLs
            if uri.host().is_some() {
                let mut parts = uri.into_parts();

                // TODO: Replace with Option::is_some_and when it's stable
                // I'm not unwrapping because then I'd have to check all of the sanctioned schemes against "".
                if let Some(scheme_str) = parts.scheme.as_ref().map(|scheme| {
                    // URI schemes should be lowercased even though they're case insensitive. The http crate doesn't enforce this
                    // but deterministic casing makes everything easier.
                    let scheme = scheme.as_str().to_lowercase();

                    // Only allow a liberal subset of schemes.
                    if SANCTIONED_SCHEMES.contains(&scheme.as_str()) {
                        Ok(scheme)
                    } else {
                        Err(DeErrorTrait::custom(format!("invalid scheme: `{scheme}`",)))
                    }
                }) {
                    let scheme_str = scheme_str?;
                    parts.scheme = Some(scheme_str.as_str().try_into().unwrap());

                    // Cleaned and sanctioned scheme, absolute URI
                    Ok(UriWrapper(parts.try_into().unwrap()))
                } else {
                    // No scheme, absolute URI
                    Ok(UriWrapper(parts.try_into().unwrap()))
                }
            } else {
                Err(DeErrorTrait::custom("relative URL without a base"))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::UriWrapper;
    use serde_test::{assert_de_tokens_error, assert_tokens, Token};

    const EXAMPLE_OK: &str = "https://example.com/";
    const EXAMPLE_BAD: &str = "/etc/shadow";
    const EXAMPLE_TRACKER: &str = "udp://somefakesitemeow.faketld:666/announce";
    const EXAMPLE_FILE: &str = "file://home/joshua/Documents/Essays/why_i_like_mudkips.tex";

    #[test]
    fn uriwrapper_okay() {
        let uri = UriWrapper(EXAMPLE_OK.parse().unwrap());
        assert_tokens(&uri, &[Token::String(EXAMPLE_OK)])
    }

    #[test]
    fn uriwrapper_tracker_okay() {
        let uri = UriWrapper(EXAMPLE_TRACKER.parse().unwrap());
        assert_tokens(&uri, &[Token::String(EXAMPLE_TRACKER)])
    }

    #[test]
    fn uriwrapper_oops() {
        assert_de_tokens_error::<UriWrapper>(
            &[Token::String(EXAMPLE_BAD)],
            "relative URL without a base",
        )
    }

    #[test]
    fn uriwrapper_mudkips() {
        assert_de_tokens_error::<UriWrapper>(
            &[Token::String(EXAMPLE_FILE)],
            "invalid scheme: `file`",
        )
    }
}
