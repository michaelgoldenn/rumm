// src/gui/ThunderstoreBrowser.rs

use color_eyre::eyre::Result;
use eframe::egui::{self, Ui};
use crate::thunderstore::ModList;
use crate::mod_cache::ModCache;
use crate::user_info::Config;

/// Renders the Thunderstore Browser tab UI.
pub fn draw_thunderstore_browser(ui: &mut Ui, mod_list: &mut ModList) -> Result<()> {
    // Iterate over mods and create the UI elements.
    for new_mod in &mod_list.mods {
        ui.horizontal(|ui| {
            // Display the mod icon.
            ui.image(
                new_mod
                    .versions
                    .first()
                    .expect("there should always be a first version")
                    .icon
                    .clone(),
            );
            // Display the mod name.
            ui.label(
                new_mod
                    .versions
                    .first()
                    .expect("there should always be a first version")
                    .name
                    .clone(),
            );
            if ui.add(egui::Button::new("Add Mod")).clicked() {
                let ctx = ui.ctx().clone();
                let uuid = new_mod.uuid.to_string();
                let mod_name = new_mod.name.clone();
                let _config = Config::new();
                let new_mod_clone = new_mod.clone();
                // Clone the mod_list if needed by ModCache
                let cache_mod_list = mod_list.clone();
                let mut mod_cache = ModCache::new(&cache_mod_list);

                // Spawn an asynchronous task.
                tokio::spawn(async move {
                    match mod_cache.cache_mod_by_mod_id(&uuid, None).await {
                        Ok(_) => {
                            if mod_cache
                                .cache_mod_by_mod_id(&new_mod_clone.uuid.to_string(), None)
                                .await
                                .is_err()
                            {
                                println!("ERROR: could not enable mod: {}", mod_name);
                            }
                            // Request a repaint after the async operation completes.
                            ctx.request_repaint();
                        }
                        Err(e) => {
                            eprintln!("ERROR: could not cache mod: {}", mod_name);
                            eprintln!("{:?}", color_eyre::eyre::Report::from(e));
                        }
                    }
                });
            }
        });
    }
    Ok(())
}
