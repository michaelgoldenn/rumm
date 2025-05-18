use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use color_eyre::eyre::{Result, eyre};
use eframe::egui::Ui;

use crate::user_info::Config;

use super::TabResult;

enum ChangeType {
    RumbleDirectory(PathBuf),
    ModCacheDirectory(PathBuf),
    ConfigFile(PathBuf),
    ShouldAutoUpdate(bool),
}

pub fn draw_settings_ui(ui: &mut Ui, config: &mut Config) -> TabResult {
    let mut changes = vec![];

    ui.vertical(|ui| -> Result<()> {
        // Rumble Directory
        ui.horizontal(|ui| -> Result<()> {
            ui.label("Rumble Directory: ");
            let original_rumble_directory = config
                .rumble_directory
                .to_str()
                .ok_or(eyre!("Could not convert from rumble directory to string"))?
                .to_string()
                .clone();
            let mut rumble_directory = original_rumble_directory.clone();
            ui.text_edit_singleline(&mut rumble_directory);
            if rumble_directory != original_rumble_directory {
                changes.push(ChangeType::RumbleDirectory(rumble_directory.clone().into()));
            }
            let result = check_for_rumble_exe(&PathBuf::from_str(&rumble_directory)?);
            // If error, return error string. If Ok(false) return a static string, if Ok(true)
            let result_text = match result {
                Err(e) => format!("Error: {}", e.to_string()),
                Ok(false) => "Could not find Rumble executable in file!".into(),
                Ok(true) => "Rumble executable found!".into(),
            };
            ui.label(result_text);
            Ok(())
        });
        // Mod Cahce Directory
        ui.horizontal(|ui| -> Result<()> {
            ui.label("Mod Cache Directory: ");
            let original_cache_directory = config
                .mod_cache_directory
                .to_str()
                .ok_or(eyre!("Could not convert from cache directory to string"))?
                .to_string()
                .clone();
            let mut cache_directory = original_cache_directory.clone();
            ui.text_edit_singleline(&mut cache_directory);
            if cache_directory != original_cache_directory {
                changes.push(ChangeType::ModCacheDirectory(
                    cache_directory.clone().into(),
                ));
            };
            Ok(())
        });
        Ok(())
    });

    apply_changes(config, changes)?;
    Ok(None)
}

fn check_for_rumble_exe(path: &Path) -> Result<bool> {
    return Ok(path
        .read_dir()?
        .any(|x| x.is_ok_and(|x| x.file_name() == "RUMBLE.exe")));
}

fn apply_changes(config: &mut Config, changes: Vec<ChangeType>) -> Result<()> {
    for change in changes {
        match change {
            ChangeType::RumbleDirectory(file) => config.rumble_directory = file,
            ChangeType::ModCacheDirectory(file) => config.mod_cache_directory = file,
            ChangeType::ConfigFile(file) => config.config_file = file,
            ChangeType::ShouldAutoUpdate(x) => config.should_auto_update = x,
        }
    }
    config.save_to_file()
}
