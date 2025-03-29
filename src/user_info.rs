// handles user-related things, such as configs and enabled mods

use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::thunderstore::Mod;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub rumble_directory: PathBuf,
    pub mod_cache_directory: PathBuf,
    pub enabled_mods_file: PathBuf,
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
            enabled_mods_file: base_dir.join("enabled_mods.json"),
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
#[derive(Debug, Serialize, Deserialize)]
pub struct EnabledMods {
    ids: Vec<String>, // A list of Mod IDs to enable
}

impl EnabledMods {
    pub fn new(config: &Config) -> Self {
        let mut enabled_mods = EnabledMods { ids: vec![] };

        // Try to load the file, but only create a new one if loading fails
        // due to the file not existing
        match enabled_mods.load_from_file(&config.enabled_mods_file) {
            Ok(_) => enabled_mods,
            Err(e) => {
                // Check if the error is because the file doesn't exist
                if e.downcast_ref::<std::io::Error>().map_or(false, |io_err| {
                    io_err.kind() == std::io::ErrorKind::NotFound
                }) {
                    // Create the directory and an empty file
                    if let Some(parent) = config.enabled_mods_file.parent() {
                        fs::create_dir_all(parent).wrap_err("Failed to create parent directory");
                    }

                    // Save an empty file
                    enabled_mods
                        .save_to_file(&config.enabled_mods_file)
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
        Ok(self.ids.contains(&this_mod.id))
    }
    pub fn enable_mod(&mut self, mod_to_enable: &Mod, config: &Config) -> Result<()> {
        if !self.ids.contains(&mod_to_enable.id) {
            self.ids.push(mod_to_enable.id.clone());
        }
        self.save_to_file(&config.enabled_mods_file)?;
        Ok(())
    }
    pub fn disable_mod(&mut self, mod_to_disable: &Mod, config: &Config) -> Result<()> {
        self.ids.retain(|x| x != &mod_to_disable.id);
        self.save_to_file(&config.enabled_mods_file)?;
        Ok(())
    }
    fn save_to_file(&self, path: &Path) -> Result<()> {
        // Explicitly serialize just the ids array
        let contents = serde_json::to_string_pretty(&self.ids)?; 
        fs::write(path, contents)?;
        Ok(())
    }

    fn load_from_file(&mut self, path: &Path) -> Result<()> {
        let contents = fs::read_to_string(path)?;
        // Deserialize directly into the ids vector
        self.ids = serde_json::from_str(&contents)?;
        //println!("Loaded! Mods: {:?}", self.ids);
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
