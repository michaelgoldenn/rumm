// src/gui/LocalModList.rs

use crate::thunderstore::{Mod, ModList, Version};
use crate::user_info::LocalModOptions;
use crate::{mod_cache::ModCache, user_info::Config};
use color_eyre::eyre::{Result, eyre};
use eframe::egui::{self, Button, Checkbox, Image, Label, Ui};

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use super::{AppCommand, TabResult};

/// All data for the “Mods” tab lives here.
pub struct LocalModsTab {
    cache: ModCache,
    options: LocalModOptions,
    /// Receiver for results coming back from the worker thread.
    result_rx: Option<Receiver<Result<()>>>,
    // trying to emulate Elm with this one, might wanna switch to iced instead of egui at some point
    pending_changes: Vec<(Mod, ChangeType)>,
}

enum ChangeType {
    Enabled(bool),
    Version(String),
    RemoveVersion(Version),
    VersionLock(bool),
    Update,
    UpdateAll,
    DeleteMod,
}

impl LocalModsTab {
    pub fn new(mod_list: &ModList, options: LocalModOptions) -> Self {
        Self {
            cache: ModCache::new(mod_list),
            options,
            result_rx: None,
            pending_changes: Vec::new(),
        }
    }

    /// Draws one frame of the tab. Remains synchronous, heavy work is off‑loaded
    /// to a dedicated blocking thread so the UI never stalls.
    pub fn ui(&mut self, ui: &mut Ui) -> TabResult {
        // check whether the background thread has finished
        if let Some(rx) = &self.result_rx {
            while let Ok(r) = rx.try_recv() {
                if let Err(e) = r {
                    eprintln!("{e:?}");
                }
            }
        }

        // we need to update the cache regularly
        self.cache.update_self_from_cache();

        // collect version changes selected by the user
        let mut pending_updates: Vec<(Mod, String)> = Vec::new();
        let config = Config::new();

        egui::ScrollArea::vertical().show(ui, |ui| -> Result<()> {
            if ui.button("Update All").clicked() {
                let first = self.cache.cache_mod_list.first().ok_or(eyre!("No mods found to update!"))?;
                self.pending_changes.push((first.clone(), ChangeType::UpdateAll));
            }
            let grid_result = egui::Grid::new("Mod Grid").striped(true).show(ui, |ui| {
                for original_mod_from_thunderstore in &self.cache.cache_mod_list {
                    let mod_from_cache = match self
                        .cache
                        .prune_extra_versions_from_mod(&config, &original_mod_from_thunderstore)
                    {
                        Ok(m) => m,
                        Err(e) => return Err(e),
                    };
                    let is_mod_enabled = match self.options.is_mod_enabled(&mod_from_cache) {
                        Ok(enabled) => enabled,
                        Err(e) => return Err(e),
                    };
                    let current = self
                        .options
                        .get_mod_options_mut(original_mod_from_thunderstore.uuid.to_string())
                        .ok_or(eyre!(
                            "Mod options file not found for {}",
                            original_mod_from_thunderstore.name
                        ))?
                        .clone();

                    // Enabled checkbox
                    let mut mod_enabled_mut = is_mod_enabled.clone();
                    ui.checkbox(&mut mod_enabled_mut, "Enabled");
                    // just a hacky way to convert from the `mut bool` to the `enable/disable mod` functions
                    if mod_enabled_mut != is_mod_enabled {
                        self.pending_changes
                            .push((mod_from_cache.clone(), ChangeType::Enabled(mod_enabled_mut)));
                    }

                    if let Some(first) = original_mod_from_thunderstore.versions.first() {
                        // image
                        ui.add_enabled(is_mod_enabled, Image::new(first.icon.clone()));
                        // name
                        ui.add_enabled(is_mod_enabled, Label::new(&first.name));
                        //ui.label(&first.name);
                        //version lock checkbox
                        let mut version_lock = current.version_lock.clone();
                        let checkbox = ui.add_enabled(
                            is_mod_enabled,
                            Checkbox::new(&mut version_lock, "Lock Verison"),
                        ).on_hover_text("If checked, the mod will not update unless you use the version selector to the right");
                        if checkbox.changed() {
                            self.pending_changes.push((
                                mod_from_cache.clone(),
                                ChangeType::VersionLock(version_lock),
                            ));
                        };
                    }
                    // version selector
                    let old_version = current.version.clone();
                    let combo_box =
                        egui::ComboBox::from_id_salt(original_mod_from_thunderstore.uuid)
                            .selected_text(&current.version);
                    // wrap combobox in enabled check to let it be disabled when auto-updates are off
                    let mut selected_version = current.version.clone();
                    ui.add_enabled_ui(
                        self.options
                            .get_version_lock(&mod_from_cache.uuid)
                            .unwrap_or(false)
                            && is_mod_enabled,
                        |ui| {
                            combo_box.show_ui(ui, |ui| {
                                for v in &original_mod_from_thunderstore.versions {
                                    ui.horizontal(|ui| {
                                        ui.selectable_value(
                                            &mut selected_version,
                                            v.version_number.clone(),
                                            &v.version_number,
                                        );
                                        // show delete button if the mod is available locally
                                        if mod_from_cache
                                            .versions
                                            .iter()
                                            .any(|x| x.version_number == v.version_number)
                                        {
                                            if ui
                                                .add(Button::image(egui::include_image!(
                                                    "./icons/trash.svg"
                                                )))
                                                .on_hover_text("Delete version")
                                                .clicked()
                                            {
                                                //self.cache.remove_old_versions_from_cache(&config, &mod_from_cache);
                                                self.pending_changes.push((
                                                    mod_from_cache.clone(),
                                                    ChangeType::RemoveVersion(v.clone()),
                                                ));
                                            };
                                        }
                                    });
                                }
                            });
                        },
                    );
                    if old_version != selected_version {
                        pending_updates.push((mod_from_cache.clone(), selected_version.clone()));
                        self.pending_changes.push((
                            mod_from_cache.clone(),
                            ChangeType::Version(selected_version),
                        ));
                    }
                    if ui.button("Update").clicked() {
                        self.pending_changes.push((mod_from_cache.clone(), ChangeType::Update));
                    }
                    // Delete Button
                    if ui
                        .add(Button::image(egui::include_image!("./icons/trash.svg")))
                        .on_hover_text("Delete mod")
                        .clicked()
                    {
                        self.pending_changes
                            .push((mod_from_cache.clone(), ChangeType::DeleteMod));
                    };
                    ui.end_row();
                }
                Ok(())
            });

            // Handle any errors from the grid
            if let Err(e) = grid_result.inner {
                return Err(e);
            }

            Ok(()) // Add this to return Ok(()) from the ScrollArea closure
        });

        // start the worker if we have a job for it
        if !pending_updates.is_empty() {
            let (tx, rx): (Sender<Result<()>>, Receiver<Result<()>>) = mpsc::channel();
            self.result_rx = Some(rx);

            // Move the necessary data into the thread. Clone or Arc‑wrap as needed.
            let mut cache = self.cache.clone();
            let mut options = self.options.clone();

            thread::spawn(move || {
                for (m, v) in pending_updates {
                    let res = change_mod_version_blocking(&mut cache, &mut options, m, v);
                    let _ = tx.send(res);
                }
            });
        }
        self.update_state()
        //todo!()
    }
    fn update_state(&mut self) -> Result<Option<AppCommand>> {
        let config = Config::new();
        let mut mod_options = LocalModOptions::new(&config);
        // I now realize there can only be one change per frame (user can't click two buttons on the same frame!) so this is redundant
        for change in &self.pending_changes {
            let (mod_to_update, change_type) = change;
            println!("Updating State!");
            match change_type {
                ChangeType::Enabled(enabled) => {
                    { mod_options.set_mod_enabled(&mod_to_update, &config, *enabled) }?
                }
                ChangeType::VersionLock(enabled) => {
                    { mod_options.set_version_lock(&mod_to_update.uuid, *enabled, &config) }?
                }
                ChangeType::Version(version) => {
                    mod_options.set_mod_version(&mod_to_update.uuid, &version.to_string(), &config)
                }?,
                ChangeType::RemoveVersion(version) => {
                    self.cache.remove_version_from_cache(
                        &config,
                        mod_to_update,
                        version.clone(),
                    )?;
                }
                ChangeType::DeleteMod => {
                    self.cache.remove_mod_from_cache(&config, mod_to_update)?;
                    //return Ok(Some(AppCommand))
                }
                ChangeType::Update => {
                    //self.cache.update_mod(&config, mod_to_update);
                    let update = mod_to_update.clone();
                    self.pending_changes.clear();
                    return Ok(Some(AppCommand::UpdateMod(update)))
                }
                ChangeType::UpdateAll => {
                    self.pending_changes.clear();
                    return Ok(Some(AppCommand::UpdateAllMods));
                }
            }
        }
        self.pending_changes.clear();
        // update state
        self.options = mod_options;
        Ok(None)
    }
}

/// runs inside the background thread
fn change_mod_version_blocking(
    cache: &mut ModCache,
    options: &mut LocalModOptions,
    mod_to_change: Mod,
    new_version: String,
) -> Result<()> {
    // If the cache API is async, we create a short‑lived Tokio runtime and block on it.
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| eyre!("failed to create a Tokio runtime: {e}"))?;

    let config = Config::new();
    rt.block_on(async {
        println!(
            "Checking version: {}, mod versions: {}",
            &new_version,
            &mod_to_change.versions.len()
        );
        match cache.does_mod_have_version(&mod_to_change, &new_version) {
            None | Some(false) => {
                println!("Adding mod to cache!");
                cache
                    .cache_mod_by_mod_id(&mod_to_change.uuid.to_string(), Some(&new_version))
                    .await?;
            }
            Some(true) => {
                println!(
                    "Mod Already in cache! Versions: {:?}",
                    mod_to_change
                        .versions
                        .iter()
                        .map(|x| x.version_number.clone())
                        .collect::<Vec<String>>()
                );
                //options.set_mod_version(&mod_to_change.uuid, new_version, &config);
            }
        }
        Ok(())
    })
}
