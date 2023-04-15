# BERT Encodes Rusty Torrents

`BERT` is a recursive descent parser (I think?) for Bencode that is implemented using [nom](https://github.com/rust-bakery/nom).

# Acknowledgements
- [BEP-0003](https://www.bittorrent.org/beps/bep_0003.html): BitTorrent specification for Bencode.
- [Creating a bencode parser with nom](https://edgarluque.com/blog/bencode-parser-with-nom/): Edgar Luque's tutorial on writing a Bencode parser using nom.
- [Wikipedia](https://en.wikipedia.org/wiki/Bencode)

# Similar projects
- [bendy](https://github.com/P3KI/bendy): Bencode marshalling that focusses on correctness.
- [intermodal](https://github.com/casey/intermodal): Torrent file suite.
- [rqbit](https://github.com/ikatson/rqbit): BitTorrent client written in Rust. Uses its own Bencode parser.
- [serde-bencode](https://github.com/toby/serde-bencode): Bencode deserializer that uses Serde. Unmaintained.
