use color_eyre::eyre::{Result, eyre};
use mod_cache::ModCache;
use user_info::Config;
use std::{env, time::Duration};

mod egui;
mod mod_cache;
mod thunderstore;
mod updater;
mod user_info;

use egui::start_gui;
use thunderstore::ModList;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let list = ModList::new().await?;
    if env::args().any(|x| x == "--updater") {
        update_loop().await
    } else {
        match start_gui(list) {
            Ok(_) => Ok(()),
            Err(e) => Err(eyre!(e.to_string())),
        }
    }
}

async fn update_loop() -> Result<()> {
    loop {
        //update
        let config = Config::new();
        let mod_list = &ModList::new().await?;
        let mut cache = ModCache::new(mod_list);
        cache.update_all_mods(&config).await?;
        

        // sleep
        let sleep_minutes = 30;
        tokio::time::sleep(Duration::from_secs(sleep_minutes*60)).await;
    }
}