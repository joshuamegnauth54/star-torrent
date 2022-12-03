//! Representations of URIs (Uniform Resource Identifiers) as they're used in torrents for trackers, HTTP seeds, Kademlia, et cetera.

pub mod node;
pub mod uriwrapper;

pub use uriwrapper::UriWrapper;
pub use node::Node;
