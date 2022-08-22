use bedit_torrent::Torrent;
use std::{env, fs};

#[test]
fn test_files() {
    let path = format!(
        "{}/{}",
        env::var("CARGO_MANIFEST_DIR").unwrap(),
        "resources/tests/"
    );

    for entry in fs::read_dir(path).expect("Test torrent files not found.") {
        let entry = entry.unwrap();
        let contents = fs::read(entry.path()).unwrap();
        _ = Torrent::de_from_bytes(&contents)
            .unwrap_or_else(|_| panic!("Torrent file failed to deserialize: {:?}", entry.path()));
    }
}
