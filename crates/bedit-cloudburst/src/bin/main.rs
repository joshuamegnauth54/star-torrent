use bedit_cloudburst::Torrent;
use color_eyre::Result;
use serde_bencode::Error;
use std::{
    error::Error as StdError,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

fn torrent_from_file(path: &Path) -> Result<Torrent, Error> {
    let mut torrent = BufReader::new(File::open(path).map_err(|error| {
        Error::Custom(format!(
            "Unable to open file: {}\nWith error: {error}",
            path.display()
        ))
    })?);

    let mut buffer = Vec::new();
    torrent.read_to_end(&mut buffer).map_err(|error| {
        Error::Custom(format!(
            "Failed to read torrent: {}\nWith error: {error}",
            path.display()
        ))
    })?;

    serde_bencode::from_bytes(&buffer)
}

fn torrent_directory(path: &Path) -> Result<Vec<Result<Torrent, Error>>, Error> {
    path.read_dir()
        .map_err(|error| {
            Error::Custom(format!(
                "Failed to read directory {}\nWith error: {error}",
                path.display()
            ))
        })?
        .filter_map(|maybe_entry| {
            maybe_entry
                .map_err(|error| {
                    Error::Custom(format!(
                        "Failed to read directory entry\nWith error: {error}"
                    ))
                })
                // Filter to remove any non-torrent entries and return the deserialization result.
                .map(|entry| {
                    let path = entry.path();
                    if path.is_file()
                        && path.extension().and_then(|ext| ext.to_str()) == Some("torrent")
                    {
                        Some(torrent_from_file(&path))
                    } else {
                        None
                    }
                })
                .transpose()
        })
        .collect()
}

fn print_torrent_result(result: Result<Torrent, Error>, path: &Path) {
    match result {
        Ok(torrent) => {
            println!("Deseralized torrent: {}", torrent.name());
        }
        Err(e) => eprintln!(
            "Torrent failed to deserialize: {path:?}\nError: {e}, Error source: {}",
            e.source()
                .map_or_else(|| "No source".to_string(), |e| e.to_string())
        ),
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();

    for arg in std::env::args().skip(1) {
        let path = Path::new(&arg);
        if path.is_file() {
            print_torrent_result(torrent_from_file(path), path);
        } else {
            for torrent in torrent_directory(path)? {
                print_torrent_result(torrent, path)
            }
        }
    }

    Ok(())
}
