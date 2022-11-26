#![feature(once_cell)]

use color_eyre::owo_colors::{OwoColorize, Style};
use color_eyre::{eyre::Context, Report, Result};
use star_cloudburst::Torrent;
use std::{
    //cell::OnceCell,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

/*const OK: OnceCell<color_eyre::owo_colors::Styled<&str>> =
    Style::new().bright_green().style("Ok").into();
*/

/// Deserialize torrent files.
#[derive(argh::FromArgs)]
struct Args {
    /// parse torrents as a map for debugging purposes
    #[argh(switch, short = 'm')]
    map: bool,
    /// paths to torrent files and/or directories of torrent files
    #[argh(positional)]
    torrents: Vec<PathBuf>,
}

fn torrent_from_file(path: &Path) -> Result<Vec<u8>, Report> {
    let mut torrent = BufReader::new(
        File::open(path)
            .wrap_err_with(|| format!("Unable to open file: {}", path.display().blue()))?,
    );

    let mut buffer = Vec::new();
    torrent
        .read_to_end(&mut buffer)
        .wrap_err_with(|| (format!("Failed to read torrent: {}", path.display().blue())))?;
    Ok(buffer)
}

// Filters for .torrent files.
fn torrent_directory(path: &Path) -> Result<Vec<PathBuf>, Report> {
    path.read_dir()
        .wrap_err_with(|| (format!("Failed to read directory: {}", path.display().blue())))?
        .filter_map(|maybe_entry| {
            maybe_entry
                .wrap_err_with(|| {
                    format!(
                        "Failed to read directory entry at: {}",
                        path.display().blue()
                    )
                })
                // Filter to remove any non-torrent entries and return the deserialization result.
                .map(|entry| {
                    let path = entry.path();
                    if path.is_file()
                        && path.extension().and_then(|ext| ext.to_str()) == Some("torrent")
                    {
                        Some(path)
                    } else {
                        None
                    }
                })
                .transpose()
        })
        .collect()
}

fn print_torrents(torrent_paths: &[PathBuf]) {
    let ok = Style::new().bright_green().style("Ok");
    let err = Style::new().red().style("Err");
    let error = Style::new().bright_red();

    for path in torrent_paths {
        match torrent_from_file(path) {
            Ok(buffer) => {
                match serde_bencode::from_bytes::<Torrent>(&buffer).wrap_err_with(|| {
                    format!("Torrent failed to deserialize: {}", path.display().blue())
                }) {
                    Ok(torrent) => println!("[{ok}] => {}", torrent.name()),
                    Err(e) => eprintln!("[{err}] => {}", error.style(e)),
                }
            }
            Err(e) => eprintln!("[{err}] => {:#}", error.style(e)),
        }
    }
}

fn deserialize_as_map(torrents: &[PathBuf]) {
    for path in torrents {
        match torrent_from_file(&path) {
            Ok(buffer) => {}
            Err(e) => {}
        }
    }
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    color_eyre::install()?;
    pretty_env_logger::init();

    // Flatten directories and single paths into a vector of paths.
    let torrents: Vec<_> = args
        .torrents
        .into_iter()
        .flat_map(|path| {
            // An iterator that yields one file if it's a file or a stream of files if it's a dir.
            // The result is flattened into an iterator of paths that's collected into a vector.
            if path.is_file() {
                // I'm sorry.
                Ok(vec![path])
            } else {
                torrent_directory(&path)
            }
        })
        .flatten()
        .collect();

    if args.map {
        deserialize_as_map(&torrents)
    } else {
        print_torrents(&torrents)
    }

    Ok(())
}
