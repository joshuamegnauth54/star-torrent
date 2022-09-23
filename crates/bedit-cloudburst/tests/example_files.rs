use bedit_cloudburst::Torrent;
use std::{env, fs};

#[test]
fn test_files() {
    let path = format!(
        "{}/{}",
        env::var("CARGO_MANIFEST_DIR").expect("This test uses CARGO_MANIFEST_DIR to find the torrents directory. Run with Cargo or provide the path manually."),
        "resources/tests/"
    );

    for entry in fs::read_dir(&path).expect("Test torrent files not found.") {
        let entry =
            entry.unwrap_or_else(|error| panic!("Unable to open files at {path}\nWhy: {error}"));
        let contents = fs::read(entry.path()).unwrap_or_else(|error| {
            panic!(
                "Unable to read contents of file: {:?}\nWhy: {error}",
                entry.path()
            )
        });
        let _torrent: Torrent = serde_bencode::from_bytes(&contents).unwrap_or_else(|error| {
            panic!(
                "Torrent file failed to deserialize\nPath: {:?}\nError: {error}",
                entry.path()
            )
        });
    }
}