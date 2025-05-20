// handles user-related things, such as configs and enabled mods

use std::{
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use color_eyre::eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::thunderstore::Mod;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortType {
    Alphabetically,
    ReleaseDate,
    UpdateDate,
}
impl From<String> for SortType {
    fn from(value: String) -> Self {
        let str_value = value.as_str();
        match str_value {
            "Alphabetical" => SortType::Alphabetically,
            "Last Released" => SortType::ReleaseDate,
            "Last Updated" => SortType::UpdateDate,
            _ => panic!("Unsupported sorting type used!"),
        }
    }
}
impl From<SortType> for String {
    fn from(value: SortType) -> Self {
        match value {
            SortType::Alphabetically => "Alphabetical".to_string(),
            SortType::ReleaseDate => "Last Released".to_string(),
            SortType::UpdateDate => "Last Updated".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Will be something like /path/to/common/RUMBLE/
    pub rumble_directory: PathBuf,
    /// Will be something like /rumm/config/mod_cache/
    pub mod_cache_directory: PathBuf,
    /// Will be something like /rumm/config/enabled_mods.json
    pub config_file: PathBuf,
    pub should_auto_update: bool,
    pub thunderstore_browser_sort: SortType,
}

impl Config {
    // Constant path for the configuration file
    const CONFIG_PATH: &'static str = "config/config.json";

    pub fn new() -> Self {
        // Default configuration paths
        let base_dir = Path::new("config");

        let mut config = Self {
            rumble_directory: Config::get_rumble_directory(),
            mod_cache_directory: base_dir.join("mod_cache"),
            config_file: base_dir.join("enabled_mods.json"),
            should_auto_update: true,
            thunderstore_browser_sort: SortType::UpdateDate,
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

    // These functions were inspired by / adapted from xLoadingx's work
    // https://github.com/xLoadingx/Rumble-Mod-Manager/blob/05d827240f4a5535c243954da86b731dceec1231/Rumble%20Mod%20Manager/LaunchPage.cs#L860
    // with modifications to account for linux use
    /// WARNING - Path generated is not guaranteed to be correct
    fn get_rumble_directory() -> PathBuf {
        #[cfg(target_os = "windows")] {
            // Windows
            let steam_path = Config::find_steam_path_windows()
                .unwrap_or_else(|| PathBuf::from("Rumble path not found, replace this"));
            let rumble_path = steam_path.join("steamapps").join("common").join("RUMBLE");
            rumble_path
        }
        #[cfg(not(target_os = "windows"))]
        {
            // Linux
            let steam_path = Config::find_steam_path_linux()
                .unwrap_or_else(|| PathBuf::from("Rumble path not found, replace this"));
            let rumble_path = steam_path.join("steamapps").join("common").join("RUMBLE");
            rumble_path
        }
    }
    /// ## WARNING - AI Generated (I checked it though)
    /// Finds the Steam installation path on Linux
    #[cfg(not(target_os = "windows"))]
    fn find_steam_path_linux() -> Option<PathBuf> {
        // Check HOME environment variable for user-specific paths
        if let Ok(home) = env::var("HOME") {
            // Common user-specific Steam installation paths
            let paths = [
                format!("{}/.steam/steam", home),
                format!("{}/.local/share/Steam", home),
                format!("{}/.steam/debian-installation", home),
                format!("{}/.steam/root", home),
            ];
            // Check each path for existence
            for path in paths.iter() {
                let path = Path::new(path);
                if path.exists() && path.is_dir() {
                    return Some(path.to_path_buf());
                }
            }
        }
        // Check system-wide installation path
        let system_paths = ["/usr/lib/steam", "/usr/lib64/steam", "/usr/local/lib/steam"];
        for path in system_paths.iter() {
            let path = Path::new(path);
            if path.exists() && path.is_dir() {
                return Some(path.to_path_buf());
            }
        }
        // Check the STEAM_ROOT environment variable if set
        if let Ok(steam_root) = env::var("STEAM_ROOT") {
            let path = Path::new(&steam_root);
            if path.exists() && path.is_dir() {
                return Some(path.to_path_buf());
            }
        }

        None
    }

    #[cfg(target_os = "windows")]
    /// Finds the Steam installation path on Windows
    pub fn find_steam_path_windows() -> Option<PathBuf> {
        // try to get the path from the registry
        let result = (|| -> io::Result<PathBuf> {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let steam_key = hkcu.open_subkey("SOFTWARE\\Valve\\Steam")?;
            let install_path: String = steam_key.get_value("SteamPath")?;
            Ok(PathBuf::from(install_path))
        })();

        if let Ok(path) = result {
            if path.exists() && path.is_dir() {
                return Some(path);
            }
        }
        return None;
    }
}
/// The options for a mod - if it's enabled, what it's version is, etc.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModOptions {
    pub id: String,
    pub version: String,
    pub version_lock: bool,
    pub enabled: bool,
}
impl PartialEq for ModOptions {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Stores options for downloaded mods, lets you enable the mod, set the version, etc. -
#[derive(Debug, Serialize, Deserialize, Clone)]
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
                        .save_to_file(&config)
                        .wrap_err("Failed to create enabled mods file");
                } else {
                    // For other types of errors, return the original error
                    println!("Error: {}", e);
                }

                enabled_mods
            }
        }
    }
    fn get_mod(&self, id: Uuid) -> Option<&ModOptions> {
        self.mods.iter().find(|x| x.id == id.to_string())
    }
    fn get_mod_mut(&mut self, id: Uuid) -> Option<&mut ModOptions> {
        self.mods.iter_mut().find(|x| x.id == id.to_string())
    }

    pub fn is_mod_enabled(&self, this_mod: &Mod) -> Result<bool> {
        Ok(self
            .get_mod_options(this_mod.uuid.to_string())
            .map_or(false, |opt| opt.enabled))
    }

    pub fn enable_mod(&mut self, mod_to_enable: &Mod, config: &Config) -> Result<()> {
        let mod_id = mod_to_enable.uuid.to_string();

        match self.get_mod_options_mut(mod_id.clone()) {
            Some(mod_option) => mod_option.enabled = true,
            None => {
                let mod_options = ModOptions {
                    id: mod_id,
                    version: mod_to_enable
                        .versions
                        .first()
                        .expect("mods should always have at least one version")
                        .version_number
                        .clone(),
                    version_lock: false,
                    enabled: true,
                };
                self.mods.push(mod_options);
            }
        }

        self.save_to_file(&config)?;
        Ok(())
    }

    pub fn disable_mod(&mut self, mod_to_disable: &Mod, config: &Config) -> Result<()> {
        if let Some(mod_option) = self.get_mod_options_mut(mod_to_disable.uuid.to_string()) {
            mod_option.enabled = false;
        }
        self.save_to_file(&config)?;
        Ok(())
    }

    pub fn set_mod_enabled(
        &mut self,
        mod_to_change: &Mod,
        config: &Config,
        enable: bool,
    ) -> Result<()> {
        match enable {
            true => self.enable_mod(mod_to_change, config),
            false => self.disable_mod(mod_to_change, config),
        }
    }

    pub fn save_to_file(&self, config: &Config) -> Result<()> {
        // Serialize the entire LocalModOptions structure
        let contents = serde_json::to_string_pretty(&self)?;
        fs::write(&config.config_file, contents)?;
        Ok(())
    }

    fn load_from_file(&mut self, path: &Path) -> Result<()> {
        let contents = fs::read_to_string(path)?;
        self.mods = serde_json::from_str::<LocalModOptions>(&contents)?.mods;
        Ok(())
    }

    // New helper methods for version management
    pub fn get_mod_options(&self, mod_id: String) -> Option<&ModOptions> {
        self.mods.iter().find(|mod_option| mod_option.id == mod_id)
    }
    pub fn get_mod_options_mut(&mut self, mod_id: String) -> Option<&mut ModOptions> {
        self.mods
            .iter_mut()
            .find(|mod_option| mod_option.id == mod_id)
    }
    pub fn set_mod_version(
        &mut self,
        mod_id: &Uuid,
        version: &String,
        config: &Config,
    ) -> Result<()> {
        if let Some(mod_option) = self.get_mod_options_mut(mod_id.to_string()) {
            mod_option.version = version.to_string();
            self.save_to_file(&config)?;
        }
        Ok(())
    }
    pub fn set_version_lock(&mut self, mod_id: &Uuid, locked: bool, config: &Config) -> Result<()> {
        if let Some(mod_option) = self.get_mod_options_mut(mod_id.to_string()) {
            mod_option.version_lock = locked;
            self.save_to_file(&config)?;
        }
        Ok(())
    }
    pub fn get_version_lock(&self, mod_id: &Uuid) -> Option<bool> {
        Some(self.get_mod_options(mod_id.to_string())?.version_lock)
    }
    pub fn get_enabled_mod_ids(&self) -> Vec<Uuid> {
        return self
            .mods
            .iter()
            .filter(|x| x.enabled)
            .map(|x| Uuid::from_str(&x.id).expect("There should only be valid UUIDs in the list"))
            .collect();
    }
}
