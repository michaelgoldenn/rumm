// src/gui/ThunderstoreBrowser.rs

use crate::thunderstore::ModList;
use eframe::egui::{self, Ui};

use super::{AppCommand, TabResult};

/// Renders the Thunderstore Browser tab UI.
pub fn draw_thunderstore_browser(ui: &mut Ui, mod_list: &mut ModList) -> TabResult {
    // Iterate over mods and create the UI elements.
    for new_mod in &mod_list.mods {
        let command = ui
            .horizontal(|ui| {
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
                    return Some(AppCommand::CacheModByID(new_mod.uuid, None));
                }
                None
            })
            .inner;
        // if anything returned a command, return it
        if let Some(cmd) = command {
            return Ok(Some(cmd));
        }
    }
    Ok(None)
}
