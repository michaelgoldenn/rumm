// src/gui/LocalModList.rs

use crate::thunderstore::ModList;
use crate::user_info::LocalModOptions;
use crate::{mod_cache::ModCache, user_info::Config};
use color_eyre::eyre::eyre;
use eframe::egui::{self, Ui};

/// Renders the Local Mod List tab UI.
pub fn draw_local_mod_list(ui: &mut Ui, mod_list: &ModList) {
    // Create a mod cache using the provided mod_list.
    let cache = ModCache::new(mod_list);
    let config = Config::new();
    let mut local_mod_options = LocalModOptions::new(&config);
    let local_mod_list = &cache.cache_mod_list;
    for this_mod in local_mod_list {
        // Get the current version selection for this mod.
        // Make sure that the type of current_version matches what you expect.
        let mut current_mod_options = local_mod_options
            .get_mod_options_mut(this_mod.uuid.to_string())
            .expect(&format!("Mod options file not found for {}", this_mod.name));

        ui.horizontal(|ui| {
            // Render the mod's icon and label.
            if let Some(first_version) = this_mod.versions.first() {
                ui.image(first_version.icon.clone());
                ui.label(first_version.name.clone()); // I LOVE CLONE() WOOOO
            }

            // Create and display the ComboBox.
            egui::ComboBox::from_label("Version")
                .selected_text(format!("{:?}", current_mod_options.version))
                .show_ui(ui, |ui| {
                    // Each item in the combobox calls selectable_value using the same mutable reference.
                    for version in &this_mod.versions {
                        ui.selectable_value(
                            &mut current_mod_options.version,
                            version.version_number.clone(),
                            &version.version_number,
                        );
                    }
                });
        });
    }
}
