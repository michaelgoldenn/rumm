// src/gui/LocalModList.rs

use crate::thunderstore::{Mod, ModList};
use crate::user_info::LocalModOptions;
use crate::{mod_cache::ModCache, user_info::Config};
use color_eyre::eyre::{Result, eyre};
use eframe::egui::{self, Ui};

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// All persistent data for the “Mods” tab lives here.
pub struct LocalModsTab {
    cache: ModCache,
    options: LocalModOptions,
    /// Receiver for results coming back from the worker thread.
    result_rx: Option<Receiver<Result<()>>>,
}

impl LocalModsTab {
    pub fn new(mod_list: &ModList, options: LocalModOptions) -> Self {
        Self {
            cache: ModCache::new(mod_list),
            options,
            result_rx: None,
        }
    }

    /// Draws one frame of the tab. Remains **synchronous**; heavy work is off‑loaded
    /// to a dedicated blocking thread so the UI never stalls.
    pub fn ui(&mut self, ui: &mut Ui) -> Result<()> {
        // ←–––––––– Check whether the background thread has finished ––––––––→
        if let Some(rx) = &self.result_rx {
            while let Ok(r) = rx.try_recv() {
                if let Err(e) = r {
                    eprintln!("{e:?}");
                }
            }
        }

        // ←–––––––– Collect version changes selected by the user ––––––––→
        let mut pending_updates: Vec<(Mod, String)> = Vec::new();

        for original_mod_from_thunderstore in &self.cache.cache_mod_list {
            let mod_from_cache = self.cache.update_versions_in_mod(&Config::new(), &original_mod_from_thunderstore)?;
            let current = self
                .options
                .get_mod_options_mut(original_mod_from_thunderstore.uuid.to_string())
                .expect(&format!("Mod options file not found for {}", original_mod_from_thunderstore.name));

            ui.horizontal(|ui| {
                if let Some(first) = original_mod_from_thunderstore.versions.first() {
                    ui.image(first.icon.clone());
                    ui.label(&first.name);
                }

                let old_version = current.version.clone();
                let response = egui::ComboBox::from_id_salt(original_mod_from_thunderstore.uuid)
                    .selected_text(&current.version)
                    .show_ui(ui, |ui| {
                        for v in &original_mod_from_thunderstore.versions {
                            ui.selectable_value(
                                &mut current.version,
                                v.version_number.clone(),
                                &v.version_number,
                            );
                        }
                    });

                if old_version != current.version {
                    pending_updates.push((mod_from_cache.clone(), current.version.clone()));
                }
            });
        }

        // ←–––––––– Kick off the worker if we have work to do ––––––––→
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
        println!("Checking version: {}, mod versions: {}", &new_version, &mod_to_change.versions.len());
        match cache.does_mod_have_version(&mod_to_change, &new_version) {
            None | Some(false) => {
                println!("Adding mod to cache!");
                cache
                    .cache_mod_by_mod_id(&mod_to_change.uuid.to_string(), Some(&new_version))
                    .await?;
            }
            Some(true) => {
                println!("Mod Already in cache! Versions: {:?}", mod_to_change.versions.iter().map(|x| x.version_number.clone()).collect::<Vec<String>>());
                options.set_mod_version(&mod_to_change.uuid.to_string(), new_version, &config);
            }
        }
        Ok(())
    })
}
