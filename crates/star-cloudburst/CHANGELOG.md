# 0.8.0
- Refactor into a more modularized style instead of re-exporting everything.
- Reimplement `Node` in terms of `UriWrapper` with non-roundtrip ser/deserialization. This allows for a consistent type instead of having to make a URI for sockets.

# 0.7.0
- Rename `benitor` to `star-torrent` and split out `benitor` into its own repo.
- Move `cloudburst`'s `lib` target into a `lib` folder.

# 0.6.0
- Simplify and improve bin tester by deduplicating code to print results and iterating over `Result<Vec<PathBuf, _>>` instead of `Result<Vec<Torrent, _>>>`.
- Add pretty colors to output.
- Use wrappers around hash types in `Info` and everywhere else.
- Implement a depth first iterator for `FileTree` and associated tests.

# 0.5.0
- Wrap `ByteBuf` into `HexBytes`
- Use `HexBytes` everywhere.
- Wrap `url::Url` because the serde implementations serialize and deserialize the `Url` `struct` rather than just a `String`.
- Apply `deny_unknown_fields` to all info dicts that aren't `Hybrid`. Serde might match the `Info` enum as version 1 or 2 rather than hybrid without it.
- Start implementing a few convenience functions for `Torrent` to make the data structure easier to use for clients.
- Switch `FileAttributes` to `serde::de::value::Error`.

# 0.4.0
- Move `PieceLength` to `pieces.rs`.
- Wrap `ByteBuf` into `Pieces`.
- Create SHA-1 wrapper types.
- Implement simple hex validation and remove hex crates.

# 0.3.0
- Roundtrip test for `FileTree`.
- Implement `PieceLength` as a wrapper around `NonZeroU64` that is valid only if the size is above 16 and a power of two.
- Only use `#[serde(deny_unknown_fields)]` when built with debug assertions. Torrent files may have exotic fields owing to the many different clients with their own needs. Failing on every randumb field is pretty bad on release.
- Speaking of which, add `url_list` to `Torrent`.
- Switch errors over to `serde_bencode::Error` because it doesn't implement `TryFrom` for the serde error I was using or whatever.
- Implement a small bin target to test random torrents.
- Rename `bedit-torrent` to `bedit-cloudburst`. I like this enough that I should name it for something that's not generic.

## 0.3.0 to do
- Strongly parse paths and the Merkel tree.
- Write tree iterators for `FileTree`.
- Use `Url` for the URLs.

# 0.2.0
- Change `Torrent::Info` to an `enum` representing different torrent meta versions. This moves validation of the versions onto the parsing that normally occurs during deserialization. Versions are now parsed during deserialization rather than validated.
- Change the `private` field of the info dicts from an `Option<bool>` to just `bool`. The extra layer of indirection is annoying, and a torrent without a `private` field can be assumed to be public (i.e. `private` is `false`).
- Move `Torrent::bool_to_int` and `Torrent::bool_from_int` to `info.rs` to reflect the changes above. Update both of them as well.
- Remove `ParseTorrentError`. `ParseTorrentError` was essentially `serde_bencode::Error` plus some unused error variants for validation.
- Remove `thiserror` because `ParseTorrentError` is gone.
- Remove `Torrent` constructor helper functions.
