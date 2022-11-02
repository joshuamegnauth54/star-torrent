use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UrlWrapper(Url);

impl<'de> Deserialize<'de> for UrlWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let url_str: Cow<'de, str> = Cow::deserialize(deserializer)?;
        Url::parse(&url_str)
            .map_err(DeError::custom)
            .map(UrlWrapper)
    }
}

impl Serialize for UrlWrapper {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl Display for UrlWrapper {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::UrlWrapper;
    use serde_test::{assert_de_tokens_error, assert_tokens, Token};

    const EXAMPLE_OK: &str = "https://example.com/";
    const EXAMPLE_BAD: &str = "/etc/shadow";

    #[test]
    fn urlwrapper_okay() {
        let url = UrlWrapper(EXAMPLE_OK.parse().unwrap());
        assert_tokens(&url, &[Token::String(EXAMPLE_OK)])
    }

    #[test]
    fn urlwrapper_oops() {
        assert_de_tokens_error::<UrlWrapper>(
            &[Token::String(EXAMPLE_BAD)],
            "relative URL without a base",
        )
    }
}
