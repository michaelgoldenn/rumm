use std::fs;
use std::path::Path;
use std::path::PathBuf;

use color_eyre::eyre::Result;
use color_eyre::eyre::eyre;
use reqwest::Url;

use crate::thunderstore::Mod;
use crate::thunderstore::ModList;
use crate::user_info::Config;

pub enum Version {
    Latest,
    NotLatest(String),
}

/// Stores local copies of mods, and handles putting mods into and out of the rumble directory
/// Each Entry is stored as `[Cache Directory]/[Mod ID]/{mod_info.json, versions/{latest, 1.0.0}}`
/// Then within each version the actual files are placed
#[derive(Debug)]
pub struct ModCache {
    /// Full mod list from thunderstore
    thunderstore_mod_list: ModList,
    /// List of mods in the cache
    cache_mod_list: Vec<Mod>,
}
impl ModCache {
    pub fn new(mod_list: ModList) -> Self {
        let mut cache = ModCache {
            thunderstore_mod_list: mod_list,
            cache_mod_list: vec![],
        };
        cache.update_mods_from_cache();
        cache
    }
    /// Adds a mod into the cache using a URL
    async fn cache_mod_by_url(url: Url, version: Version) -> Result<Mod> {
        // check if the url is valid
        todo!()
    }
    /// Adds a mod to the Rumble mods folder
    pub async fn add_mod_to_rumble_by_url(url: Url, version: Version) -> Result<Mod> {
        todo!()
    }
    /// Adds a mod to the Rumble mods folder
    pub async fn add_mod_to_rumble_by_id(id: String, version: Version) -> Result<Mod> {
        todo!()
    }
    /// Allows for reverse-searching. When given a path to a mod in the rumble folder, it will find where that mod is in the cache
    pub async fn get_mod_path_by_file_name(
        mod_file_name: &Path,
        version: Version,
    ) -> Result<PathBuf> {
        todo!()
    }
    pub fn update_mods_from_cache(&mut self) -> Result<()> {
        // spawn config to get cache folder
        let config = Config::new();
        let cache = config.mod_cache_directory.as_path();
        // search through cache, adding any mods found
        let dir = cache.read_dir()?;
        let mod_list: Vec<Result<Mod>> = dir
            .into_iter()
            .filter(|x| x.is_ok())
            .map(|x| x.expect("only ones that are ok can make it here"))
            .map(|x| ModCache::get_mod_from_dir_in_cache(&x.path()))
            .collect();
        let mut new_mod_list: Vec<Mod> = vec![];
        for new_mod in mod_list {
            new_mod_list.push(new_mod?); // want to pass along the errors and couldn't figure out how to do that in the iter
        }
        self.cache_mod_list = new_mod_list;
        Ok(())
    }
    /// returns a full mod object if given a path like: `[Cache Dir]/[Mod ID]`
    fn get_mod_from_dir_in_cache(path: &Path) -> Result<Mod> {
        if !path.is_dir() {
            return Err(eyre!(
                "Cannot get mod from a directory if it is not a directory!"
            ));
        }
        let mod_info_path = path.join("mod_info.json");
        if mod_info_path.exists() {
            let content = fs::read_to_string(mod_info_path)?;
            let response: Mod = serde_json::from_str(&content)?;
            return Ok(response);
        }
        Err(eyre!("mod_info.json not found in mod path!"))
    }
}
