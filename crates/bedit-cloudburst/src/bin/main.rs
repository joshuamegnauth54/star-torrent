use bedit_cloudburst::Torrent;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

fn torrent_from_file(path: &Path) -> Result<Torrent, serde_bencode::Error> {
    let mut torrent = BufReader::new(
        File::open(path)
            .map_err(|_| serde_bencode::Error::Custom(format!("File missing: {:?}", path)))?,
    );

    let mut buffer = Vec::new();
    torrent
        .read_to_end(&mut buffer)
        .map_err(|_| serde_bencode::Error::Custom(format!("Failed to read torrent: {:?}", path)))?;

    serde_bencode::from_bytes(&buffer)
}

fn check_torrents(path: &Path) -> Result<Vec<Torrent>, serde_bencode::Error> {
    path.read_dir()
        .map_err(|_| serde_bencode::Error::Custom("Failed to read directory".into()))?
        .filter_map(|maybe_entry| {
            maybe_entry
                .map_err(|error| {
                    serde_bencode::Error::Custom(format!("Failed to read an entry: {error}"))
                })
                .map(|entry| {
                    let path = entry.path();
                    if path.is_file() {
                        Some(torrent_from_file(&path))
                    } else {
                        None
                    }
                })
                .transpose()
        })
        .flatten()
        .collect()
}

fn main() {
    for arg in std::env::args().skip(1) {
        let path = Path::new(&arg);
        if path.is_file() {
            match torrent_from_file(path) {
                Ok(torrent) => {
                    dbg!(torrent);
                }
                Err(e) => eprintln!("Torrent failed to deserialize: {path:?}\n{e}")
            }
        }
        else if path.is_dir() {
            match check_torrents(path) {
                Ok(torrents) => {
                    for torrent in torrents {
                        dbg!(torrent);
                    }
                }
                Err(e) => eprintln!("Failed: {path:?}\n{e}")
            }
        }
    }
}
