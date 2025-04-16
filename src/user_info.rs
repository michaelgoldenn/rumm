// handles user-related things, such as configs and enabled mods

use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Context, Result};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::thunderstore::Mod;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub rumble_directory: PathBuf,
    pub mod_cache_directory: PathBuf,
    pub config_file: PathBuf,
}

impl Config {
    // Constant path for the configuration file
    const CONFIG_PATH: &'static str = "config/config.json";

    pub fn new() -> Self {
        // Default configuration paths
        let base_dir = Path::new("config");

        let mut config = Self {
            rumble_directory: Path::new("").to_path_buf(),
            mod_cache_directory: base_dir.join("mod_cache"),
            config_file: base_dir.join("enabled_mods.json"),
        };
        config.load_from_file(); // ignoring errors, if there is an error it should just use the defaults
        config
    }

    pub fn save_to_file(&self) -> Result<()> {
        // Ensure the config directory exists
        fs::create_dir_all(Path::new(Self::CONFIG_PATH).parent().unwrap())?;

        // Serialize and write the configuration
        let contents = serde_json::to_string(self)?;
        fs::write(Self::CONFIG_PATH, contents)?;
        Ok(())
    }

    pub fn load_from_file(&mut self) -> Result<()> {
        // Read and deserialize the configuration
        let contents = fs::read_to_string(Self::CONFIG_PATH)?;
        *self = serde_json::from_str(&contents)?;
        Ok(())
    }
}
/// The options for a mod - if it's enabled, what it's version is, etc.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModOptions {
    id: String,
    version: String,
    version_lock: bool,
}
impl PartialEq for ModOptions {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Stores options for downloaded mods, lets you enable the mod, set the version, etc. -
#[derive(Debug, Serialize, Deserialize)]
pub struct LocalModOptions {
    mods: Vec<ModOptions>,
}

impl LocalModOptions {
    pub fn new(config: &Config) -> Self {
        let mut enabled_mods = LocalModOptions { mods: vec![] };

        // Try to load the file, but only create a new one if loading fails
        // due to the file not existing
        match enabled_mods.load_from_file(&config.config_file) {
            Ok(_) => enabled_mods,
            Err(e) => {
                // Check if the error is because the file doesn't exist
                if e.downcast_ref::<std::io::Error>().map_or(false, |io_err| {
                    io_err.kind() == std::io::ErrorKind::NotFound
                }) {
                    // Create the directory and an empty file
                    if let Some(parent) = config.config_file.parent() {
                        fs::create_dir_all(parent).wrap_err("Failed to create parent directory");
                    }

                    // Save an empty file
                    enabled_mods
                        .save_to_file(&config.config_file)
                        .wrap_err("Failed to create enabled mods file");
                } else {
                    // For other types of errors, return the original error
                    println!("Error: {}", e);
                }

                enabled_mods
            }
        }
    }

    pub fn is_mod_enabled(&self, this_mod: &Mod) -> Result<bool> {
        // Check if any ModOptions in self.mods has matching id to this_mod's uuid
        Ok(self
            .mods
            .iter()
            .any(|mod_option| mod_option.id == this_mod.uuid.to_string()))
    }

    pub fn enable_mod(&mut self, mod_to_enable: &Mod, config: &Config) -> Result<()> {
        // Only add if not already present
        if !self
            .mods
            .iter()
            .any(|mod_option| mod_option.id == mod_to_enable.uuid.to_string())
        {
            // Create new ModOptions with default values
            let mod_options = ModOptions {
                id: mod_to_enable.uuid.to_string(),
                version: mod_to_enable
                    .versions
                    .first()
                    .expect("mods should always have at least one version")
                    .version_number
                    .clone(), // Using the mod's current version
                version_lock: false, // Default to not locked
            };
            self.mods.push(mod_options);
        }
        self.save_to_file(&config.config_file)?;
        Ok(())
    }

    pub fn disable_mod(&mut self, mod_to_disable: &Mod, config: &Config) -> Result<()> {
        // Remove any ModOptions with matching id
        self.mods
            .retain(|mod_option| mod_option.id != mod_to_disable.uuid.to_string());
        self.save_to_file(&config.config_file)?;
        Ok(())
    }

    fn save_to_file(&self, path: &Path) -> Result<()> {
        // Serialize the entire LocalModOptions structure
        let contents = serde_json::to_string_pretty(&self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    fn load_from_file(&mut self, path: &Path) -> Result<()> {
        let contents = fs::read_to_string(path)?;
        // Try to deserialize as the new format first
        match serde_json::from_str::<LocalModOptions>(&contents) {
            Ok(loaded) => {
                self.mods = loaded.mods;
                Ok(())
            }
            // If that fails, try to deserialize as the old format (array of strings)
            Err(_) => {
                let old_ids: Vec<String> = serde_json::from_str(&contents)?;

                // Convert old format to new format
                self.mods = old_ids
                    .into_iter()
                    .map(|id| ModOptions {
                        id,
                        version: String::new(), // Empty version as we don't have this info
                        version_lock: false,    // Default to not locked
                    })
                    .collect();

                Ok(())
            }
        }
    }

    // New helper methods for version management
    pub fn get_mod_options(&self, mod_id: &str) -> Option<&ModOptions> {
        self.mods.iter().find(|mod_option| mod_option.id == mod_id)
    }
    pub fn get_mod_options_mut(&mut self, mod_id: &str) -> Option<&mut ModOptions> {
        self.mods
            .iter_mut()
            .find(|mod_option| mod_option.id == mod_id)
    }
    pub fn set_mod_version(
        &mut self,
        mod_id: &str,
        version: String,
        config: &Config,
    ) -> Result<()> {
        if let Some(mod_option) = self.get_mod_options_mut(mod_id) {
            mod_option.version = version;
            self.save_to_file(&config.config_file)?;
        }
        Ok(())
    }
    pub fn set_version_lock(&mut self, mod_id: &str, locked: bool, config: &Config) -> Result<()> {
        if let Some(mod_option) = self.get_mod_options_mut(mod_id) {
            mod_option.version_lock = locked;
            self.save_to_file(&config.config_file)?;
        }
        Ok(())
    }
}

// Example function to get default path
pub fn default_save_path() -> PathBuf {
    let mut path = PathBuf::from("."); // Start with project directory
    path.push("config");
    path.push("enabled_mods.json"); // Consider .json extension
    path
}
