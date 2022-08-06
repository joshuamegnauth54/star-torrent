#[cfg(target_os = "linux")]
use dirs::config_dir;
#[cfg(target_os = "windows")]
use dirs::{data_dir, data_local_dir};

use crate::check_settings_dirs;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};

const TRANSMISSION_GUI: &str = "transmission";
const TRANSMISSION_CLI: &str = "transmission-cli";
const TRANSMISSION_DAEMON: &str = "transmission-daemon";
const TRANSMISSION_DIRS: [&str; 3] = [TRANSMISSION_GUI, TRANSMISSION_CLI, TRANSMISSION_DAEMON];
const TRANSMISSION_ENV: &str = "TRANSMISSION_HOME";

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {}

impl Settings {
    pub fn settings_dirs() -> Option<Vec<PathBuf>> {
        // Short circuit if a directory is available via env.
        if let Ok(directory) = env::var(TRANSMISSION_ENV) {
            let path: PathBuf = directory.into();
            if path.exists() {
                return Some(vec![path]);
            }
            warn!("{path:?} provided via {TRANSMISSION_ENV} but it doesn't exist.")
        }

        // Windows and Unixes have different config paths.
        cfg_if::cfg_if! {
                // Transmission on Windows writes settings to LocalAppData or Roaming.
                if #[cfg(target_os = "windows")] {
                    // The Transmission site lists data local as the main directory.
                    let settings_local = check_settings_dirs(data_local_dir()?, &TRANSMISSION_DIRS);

                    if settings_local.is_some() {
                        info!("Transmission settings found via the local data directory.");
                        settings_local
                    }
                    else {
                        info!("Checking Roaming for Transmission settings.");
                        check_settings_dirs(data_dir()?, &TRANSMISSION_DIRS)
                    }
                }
                // Linux Transmission writes to .config
                else {
                    info!("Checking .config for Transmission settings.")
                    check_settings_dirs(config_dir()?, &TRANSMISSION_DIRS);
                }
        }
    }
}
