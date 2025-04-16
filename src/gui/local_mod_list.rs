// src/gui/LocalModList.rs

use eframe::egui::{self, Ui};
use crate::mod_cache::ModCache;
use crate::thunderstore::ModList;

/// Renders the Local Mod List tab UI.
pub fn draw_local_mod_list(ui: &mut Ui, mod_list: &ModList) {
    // Create a mod cache using the provided mod_list.
    let cache = ModCache::new(mod_list);
    let local_mod_list = &cache.cache_mod_list;
    for this_mod in local_mod_list {
        ui.horizontal(|ui| {
            ui.image(
                this_mod
                    .versions
                    .first()
                    .expect("there should always be a first version")
                    .icon
                    .clone(),
            );
            ui.label(
                this_mod
                    .versions
                    .first()
                    .expect("there should always be a first version")
                    .name
                    .clone(),
            );
        });
    }
}
