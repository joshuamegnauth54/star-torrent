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