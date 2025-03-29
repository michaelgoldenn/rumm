use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use reqwest::Url;

use crate::thunderstore::Mod;
use crate::thunderstore::ModList;

/// Stores local copies of mods, and handles putting mods into and out of the rumble directory
#[derive(Debug)]
pub struct ModCache {
    /// Full mod list from thunderstore
    mod_list: ModList 
}
impl ModCache {
    pub fn new(mod_list: ModList) -> Self {
        ModCache {
            mod_list
        }
    }
    /// Adds a mod into the cache using a URL 
    async fn cache_mod_by_url(url: Url, version: Option<String>) -> Result<Mod> {
        todo!()
    }
    /// Adds a mod to the Rumble mods folder
    pub async fn add_mod_to_rumble_by_url(url: Url, version: Option<String>) {
        todo!()
    }
    pub async fn add_mod_to_rumble_by_id(id: String, version: Option<String>) {

    }
}