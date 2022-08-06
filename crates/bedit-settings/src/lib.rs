pub mod deluge;
pub mod transmission;

use log::error;
use std::path::{Path, PathBuf};

/// Check `path` for any of the settings directories.
pub(crate) fn check_settings_dirs<P>(path: P, directories: &[&str]) -> Option<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    // Filter path if it's a directory.
    let settings_dirs: Vec<_> = path
        .as_ref()
        .read_dir()
        .ok()?
        .filter_map(|entry| {
            match entry {
                Ok(entry) => {
                    let entry = entry.path();

                    // Guard against entry being a file so that a file with a name in directories isn't returned.
                    if entry.is_dir() {
                        let file_name = entry.file_name()?.to_str()?;
                        if directories.contains(&file_name) {
                            Some(entry)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Err(e) => {
                    error!(
                        "Failed reading directory entry while finding settings folders: {e}"
                    );
                    None
                }
            }
        })
        .collect();

    if settings_dirs.is_empty() {
        None
    } else {
        Some(settings_dirs)
    }
}
