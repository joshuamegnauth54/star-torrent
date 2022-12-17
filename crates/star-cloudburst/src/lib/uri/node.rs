use crate::uri::uriwrapper::UriWrapper;
use http::Uri;
use log::{debug, trace};
use serde::{
    de::{Error as DeErrorTrait, Unexpected},
    ser::Error as SerError,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::borrow::Borrow;

const AUTHORITY_INVARIANT: &str = "a valid authority from UriWrapper (this shouldn't ever happen)";
const PORT_INVARIANT: &str = "Port is missing but is canonically always available.";
const NODE_SER_TARGET: &str = "star_cloudburst::uri::Node::serialize";
const NODE_DE_TARGET: &str = "star_cloudburst::uri::Node::deserialize";

/// A [`Node`] is (host, port) pair that can be provided through DHT.
///
/// [BEP-0005](https://www.bittorrent.org/beps/bep_0005.html)
///
/// [`Node`]s are not limited to socket addresses but may also be URLs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node(UriWrapper);

impl Node {
    #[inline]
    pub fn as_uri(&self) -> &UriWrapper {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        trace!(target: NODE_DE_TARGET, "Deserializing Node");

        // Deserialize NodeTemp first which ensures a valid URI and port.
        let node_temp = NodeTemp::deserialize(deserializer)?;
        debug!(target: NODE_DE_TARGET, "Deserialized URI: {node_temp:?}");
        let NodeTemp((uri, port)) = node_temp;

        // Append port number to authority.
        let mut parts = uri.into_inner().into_parts();
        parts.authority = parts
            .authority
            .ok_or_else(|| DeErrorTrait::invalid_value(Unexpected::Option, &AUTHORITY_INVARIANT))
            .and_then(|authority| {
                Some(format!("{authority}:{port}").try_into().map_err(|e| {
                    DeErrorTrait::custom(format!(
                        "Previously valid authority is invalid after appending a port number\nUri error: {e}"
                    ))
                }))
                .transpose()
            })?;

        debug!(target: NODE_DE_TARGET, "New `Parts`: {parts:?}");

        let uri = Uri::from_parts(parts).map_err(|e| {
            DeErrorTrait::custom(format!(
                "Invalid URI despite correct `Parts`\nUri error: {e}"
            ))
        })?;
        Ok(Node(UriWrapper::from_uri_unchecked(uri)))
    }
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        trace!(target: NODE_SER_TARGET, "Serializing Node: {self:?}");

        // Split Uri into composite parts in order to serialize it as a NodeTemp.
        // NodeTemp's UriWrapper does NOT have the port embedded as per BEP-0005.
        let uri: &Uri = self.as_uri().borrow();
        let (scheme, scheme_sep) = uri
            .scheme_str()
            .map(|scheme| (scheme, "://"))
            .unwrap_or(("", ""));
        let authority = uri
            .authority()
            .ok_or_else(|| SerError::custom("URI is missing an authority."))?
            .as_str();
        let query = uri
            .path_and_query()
            .map(|query| query.as_str())
            .unwrap_or_default();

        // Extract port from Uri and port position in the authority string
        let port = uri.port().ok_or_else(|| SerError::custom(PORT_INVARIANT))?;
        // This is so I don't need to allocate a temporary String.
        let port_str = port.as_str();
        let mut port_find_iter = std::iter::once(':').chain(port_str.chars());
        let port_pos = authority
            .rfind(|ch| {
                if let Some(ch_port) = port_find_iter.next() {
                    ch == ch_port
                } else {
                    false
                }
            })
            .ok_or_else(|| SerError::custom(PORT_INVARIANT))?;

        trace!(target: NODE_SER_TARGET, "Port: {port}\nPort position: {port_pos}");

        // Recombine without port.
        let uri_temp = format!("{scheme}{scheme_sep}{authority_before}{authority_after}/{query}",
                            authority_before = &authority[0..port_pos],
                            authority_after = &authority[port_pos + port_str.len()..])
            .parse()
            .map_err(|e| SerError::custom(format!("Invalid URI after splitting and recombining `Node`; this shouldn't happen.\nUri error: {e}")))?;
        let uri_temp = UriWrapper::from_uri_unchecked(uri_temp);
        let node_temp = NodeTemp((uri_temp, port.as_u16()));

        node_temp.serialize(serializer)
    }
}

// Nodes are represented as (host, port) pairs as per [Node].
// NodeTemp is the actual type that will be deserialized and serialized while [Node] is a [UriWrapper]...wrapper.
#[derive(Deserialize, Serialize, Debug)]
struct NodeTemp((UriWrapper, u16));

#[cfg(test)]
mod tests {
    use crate::uri::node::Node;
    use serde_test::{assert_tokens, Token};

    const LOCALHOST_IP: &str = "127.0.0.1:6881";

    #[test]
    fn localhost_ip() {
        let wrapper = LOCALHOST_IP.parse().expect("URI is valid.");
        let node = Node(wrapper);
        assert_tokens(&node, &[Token::String(LOCALHOST_IP)])
    }
}
