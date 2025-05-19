use color_eyre::eyre::{Result, eyre};
use mod_cache::ModCache;
use std::{env, path::{Path, PathBuf}, str::FromStr, time::Duration};
use user_info::Config;

mod egui;
mod mod_cache;
mod thunderstore;
mod updater;
mod user_info;

use egui::start_gui;
use thunderstore::ModList;

fn main() -> Result<()> {
    color_eyre::install()?;
    // putting this here is janky, should rework in the future
    let path = PathBuf::from_str("config/thunderstore-mods.json")?;
    spawn_cached_thunderstore_response_updater(path.clone());
    
    if env::args().any(|x| x == "--updater") {
        //update_loop().await
        todo!()
    } else {
        match start_gui() {
            Ok(_) => Ok(()),
            Err(e) => Err(eyre!(e.to_string())),
        }
    }
}

// Disabled for now, eventually will make it so that it can auto-update mods in the background
/* async fn update_loop() -> Result<()> {
    loop {
        //update
        let config = Config::new();
        let mod_list = &ModList::new()?;
        let mut cache = ModCache::new(mod_list);
        cache.update_all_mods(&config).await?;

        // sleep
        let sleep_minutes = 30;
        tokio::time::sleep(Duration::from_secs(sleep_minutes * 60)).await;
    }
} */

/// Automatically stores a response from thunderstore to make startups faster (plus can still see mods even when offline)
fn spawn_cached_thunderstore_response_updater(cache_path: PathBuf) {
    std::thread::spawn(move || {
        loop {
            match fetch_thunderstore_response() {
                Ok(list) => {
                    if let Err(e) = std::fs::write(&cache_path, serde_json::to_vec(&list).unwrap())
                    {
                        eprintln!("cannot write cache: {e}");
                    }
                }
                Err(e) => eprintln!("refresh failed: {e}"),
            }
            std::thread::sleep(std::time::Duration::from_secs(60 * 60)); // every hour
        }
    });
}
fn fetch_thunderstore_response() -> color_eyre::Result<ModList> {
    let url = "https://thunderstore.io/c/rumble/api/v1/package/";
    Ok(ModList {
        mods: reqwest::blocking::get(url)?.json()?,
    })
}
