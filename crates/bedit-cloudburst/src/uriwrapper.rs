use http::uri::Uri;
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UriWrapper(Uri);

impl<'de> Deserialize<'de> for UriWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let uri_str: Cow<'de, str> = Cow::deserialize(deserializer)?;
        uri_str
            .parse()
            .map_err(DeError::custom)
            .and_then(|uri: Uri| {
                if uri.host().is_some() {
                    Ok(UriWrapper(uri))
                } else {
                    Err(DeError::custom("relative URL without a base"))
                }
            })
    }
}

impl Serialize for UriWrapper {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl Display for UriWrapper {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Uri as Display>::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::UriWrapper;
    use serde_test::{assert_de_tokens_error, assert_tokens, Token};

    const EXAMPLE_OK: &str = "https://example.com/";
    const EXAMPLE_BAD: &str = "/etc/shadow";
    const EXAMPLE_TRACKER: &str = "udp://somefakesitemeow.faketld:666/announce";

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
}
