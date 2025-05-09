// src/gui/LocalModList.rs

use crate::thunderstore::{Mod, ModList};
use crate::user_info::LocalModOptions;
use crate::{mod_cache::ModCache, user_info::Config};
use color_eyre::eyre::{Result, eyre};
use eframe::egui::{self, Ui};

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

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
    VersionLock(bool),
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
    pub fn ui(&mut self, ui: &mut Ui) -> Result<()> {
        // check whether the background thread has finished
        if let Some(rx) = &self.result_rx {
            while let Ok(r) = rx.try_recv() {
                if let Err(e) = r {
                    eprintln!("{e:?}");
                }
            }
        }

        // collect version changes selected by the user
        let mut pending_updates: Vec<(Mod, String)> = Vec::new();
        let config = Config::new();

        /* // Debug prints all of the enabled mods each frame
        println!(
            "Enabled Mods: {:?}",
            local_mod_options
                .get_enabled_mod_ids()
                .iter()
                .map(|x| self
                    .cache
                    .cache_mod_list
                    .iter()
                    .find(|y| y.uuid == *x)
                    .map(|z| z.name.clone())
                    .or(Some(String::from("Unknown")))
                    .unwrap())
                .collect::<Vec<String>>()
        ); */

        for original_mod_from_thunderstore in &self.cache.cache_mod_list {
            let mod_from_cache = self
                .cache
                .update_versions_in_mod(&config, &original_mod_from_thunderstore)?;
            let is_mod_enabled = self.options.is_mod_enabled(&mod_from_cache)?;
            let current = self
                .options
                .get_mod_options_mut(original_mod_from_thunderstore.uuid.to_string())
                .expect(&format!(
                    "Mod options file not found for {}",
                    original_mod_from_thunderstore.name
                ))
                .clone();

            ui.horizontal(|ui| {
                if let Some(first) = original_mod_from_thunderstore.versions.first() {
                    // image
                    ui.image(first.icon.clone());
                    // name
                    ui.label(&first.name);
                    //version lock checkbox
                    let mut mod_version_lock = current.version_lock.clone();
                    let checkbox = ui.checkbox(&mut mod_version_lock, "version locked");
                    if checkbox.changed() {
                        self.pending_changes.push((
                            mod_from_cache.clone(),
                            ChangeType::VersionLock(mod_version_lock),
                        ));
                    };
                }
                // version selector
                let old_version = current.version.clone();
                let combo_box = egui::ComboBox::from_id_salt(original_mod_from_thunderstore.uuid)
                    .selected_text(&current.version);
                // wrap combobox in enabled check to let it be disabled when auto-updates are off
                let mut selected_version = current.version.clone();
                ui.add_enabled_ui(
                    self.options
                        .get_version_lock(&mod_from_cache.uuid)
                        .unwrap_or(false),
                    |ui| {
                        combo_box.show_ui(ui, |ui| {
                            for v in &original_mod_from_thunderstore.versions {
                                ui.selectable_value(
                                    &mut selected_version,
                                    v.version_number.clone(),
                                    &v.version_number,
                                );
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
                if let Some(first) = original_mod_from_thunderstore.versions.first() {
                    let mut mod_enabled_mut = is_mod_enabled.clone();
                    ui.checkbox(&mut mod_enabled_mut, "Enabled");
                    // just a hacky way to convert from the `mut bool` to the `enable/disable mod` functions
                    if mod_enabled_mut != is_mod_enabled {
                        self.pending_changes
                            .push((mod_from_cache.clone(), ChangeType::Enabled(mod_enabled_mut)));
                    }
                }
            });
        }

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
        self.update_state()?;
        Ok(())
    }
    fn update_state(&mut self) -> Result<()> {
        let config = Config::new();
        let mut mod_options = LocalModOptions::new(&config);
        for change in &self.pending_changes {
            let (mod_stuff, change_type) = change;
            println!("Updating State!");
            match change_type {
                ChangeType::Enabled(enabled) => {
                    { mod_options.set_mod_enabled(&mod_stuff, &config, *enabled) }?
                }
                ChangeType::VersionLock(enabled) => {
                    { mod_options.set_version_lock(&mod_stuff.uuid, *enabled, &config) }?
                }
                ChangeType::Version(version) => {
                    { mod_options.set_mod_version(&mod_stuff.uuid, version.to_string(), &config) }?
                }
            }
        }
        self.pending_changes.clear();
        // update state
        self.options = mod_options;
        Ok(())
    }
}

/// Runs inside the background thread. It blocks, but that is fine off the UI thread.
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
