
use color_eyre::eyre::{eyre, Result};

mod egui;
mod thunderstore;
mod user_info;
mod mod_cache;
mod updater;

use thunderstore::ModList;
use egui::start_gui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let list = ModList::new().await?;
    match start_gui(list) {
        Ok(a) => Ok(()),
        Err(e) => Err(eyre!(e.to_string()))
    }
}