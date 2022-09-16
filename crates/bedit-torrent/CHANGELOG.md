# 0.2.0
- Change `Torrent::Info` to an `enum` representing different torrent meta versions. This moves validation of the versions onto the parsing that normally occurs during deserialization. Versions are now parsed during deserialization rather than validated.
- Change the `private` field of the info dicts from an `Option<bool>` to just `bool`. The extra layer of indirection is annoying, and a torrent without a `private` field can be assumed to be public (i.e. `private` is `false`).
- Move `Torrent::bool_to_int` and `Torrent::bool_from_int` to `info.rs` to reflect the changes above. Update both of them as well.
- Remove `ParseTorrentError`. `ParseTorrentError` was essentially `serde_bencode::Error` plus some unused error variants for validation.
- Remove `thiserror` because `ParseTorrentError` is gone.
- Remove `Torrent` constructor helper functions.
