// src/gui/ThunderstoreBrowser.rs

use std::{path::PathBuf, str::FromStr};

use crate::{
    config_and_such::{Config, SortType},
    thunderstore::ModList,
};
use eframe::egui::{self, ComboBox, Ui};

use super::{AppCommand, TabResult};

/// Renders the Thunderstore Browser tab UI
pub fn draw_thunderstore_browser(ui: &mut Ui) -> TabResult {
    let mut mod_list = ModList::new(PathBuf::from_str("config/thunderstore-mods.json").unwrap())?;
    let combo_box = ComboBox::from_id_salt("test");
    let mut config = Config::new();
    let mut sort = config.thunderstore_browser_sort.clone();
    let sorted_mod_list = mod_list.sort(&sort).clone();
    let mut selected_sort = String::from(config.thunderstore_browser_sort);
    combo_box
        .selected_text(String::from(sort.clone()))
        .show_ui(ui, |ui| {
            let sort_options = vec![
                SortType::Alphabetically,
                SortType::UpdateDate,
                SortType::ReleaseDate,
            ];
            for option in sort_options.clone() {
                ui.selectable_value(
                    &mut selected_sort,
                    option.clone().into(),
                    String::from(option),
                );
            }
        });
    let selected_sort_type = SortType::from(selected_sort);
    if sort != selected_sort_type {
        config.thunderstore_browser_sort = selected_sort_type;
        config.save_to_file()?;
    }
    let command = egui::Grid::new("Mod Grid")
        .striped(true)
        .show(ui, |ui| -> Option<AppCommand> {
            for new_mod in &sorted_mod_list.mods {
                // mod icon
                ui.image(
                    new_mod
                        .versions
                        .first()
                        .expect("mods should always have a first version")
                        .icon
                        .clone(),
                );
                // mod name
                ui.label(
                    new_mod
                        .versions
                        .first()
                        .expect("mods should always have a first version")
                        .name
                        .clone(),
                );
                if ui.add(egui::Button::new("Add Mod")).clicked() {
                    return Some(AppCommand::CacheModByID(new_mod.uuid, None));
                }
                ui.end_row();
            }
            None
        })
        .inner;
    if let Some(cmd) = command {
        return Ok(Some(cmd));
    }
    Ok(None)
}
