use std::{path::PathBuf, str::FromStr};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
/// Represents a single remap entry in the configuration file.
pub struct RemapEntry {
    /// The source key code to be remapped.
    pub from: String,
    /// The target key code to which the source key code will be remapped.
    pub to: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
/// Represents the configuration for the keymapper application.
pub struct Config {
    /// Path to the evdev input device (e.g., /dev/input/event0).
    pub input_device: String,
    /// Remap table for keymapper.
    pub remap: Vec<RemapEntry>,
}

impl Config {
    /// Returns the remap table as a vector of tuples containing the source and target key codes.
    pub fn remap_table(&self) -> Vec<(evdev::KeyCode, evdev::KeyCode)> {
        self.remap
            .iter()
            .filter_map(|entry| {
                let from_key = evdev::KeyCode::from_str(&entry.from);
                let to_key = evdev::KeyCode::from_str(&entry.to);
                match (from_key, to_key) {
                    (Ok(from), Ok(to)) => Some((from, to)),
                    _ => None,
                }
            })
            .collect()
    }
}

const DEFAULT_CONFIG: &str = "\
# Configuration for keymapper.

# Path to the evdev input device (e.g., /dev/input/event0).
input_device = \"/dev/input/event0\"

# Remap table for keymapper.
[[remap]]
from = \"KEY_A\"
to = \"KEY_B\"

[[remap]]
from = \"KEY_CAPSLOCK\"
to = \"KEY_ESC\"
";

/// Returns the path to the configuration file.
pub fn config_path() -> PathBuf {
    let mut path = dirs::config_dir().expect("Failed to get config directory");
    path.push("keymapper");
    path.push("config.toml");
    path
}

/// Loads the configuration from the configuration file. If the file does not exist, it will be created with default values.
pub fn load_config() -> Config {
    let path = config_path();
    if !path.exists() {
        std::fs::create_dir_all(path.parent().unwrap()).expect("Failed to create config directory");
        std::fs::write(&path, DEFAULT_CONFIG).expect("Failed to write default config");
    }

    let config_content = std::fs::read_to_string(&path).expect("Failed to read config file");
    toml::from_str(&config_content).expect("Failed to parse config file")
}

/// Tries to load the configuration from the configuration file. Returns `None` if the file does not exist or cannot be read.
pub fn try_load_config() -> Option<Config> {
    let path = config_path();
    if !path.exists() {
        return None;
    }

    let config_content = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&config_content).ok()
}

/// Loads the remap table from the configuration file.
pub fn load_remap_table() -> Vec<(evdev::KeyCode, evdev::KeyCode)> {
    let config = load_config();
    config.remap_table()
}
