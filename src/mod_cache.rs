use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use async_recursion::async_recursion;
use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use color_eyre::eyre::eyre;
use rust_search::SearchBuilder;
use uuid::Uuid;

use crate::thunderstore::Mod;
use crate::thunderstore::ModList;
use crate::user_info::Config;
use crate::user_info::LocalModOptions;

/// Stores local copies of mods, and handles putting mods into and out of the rumble directory.
/// Each Entry is stored as `[Cache Directory]/[Mod ID]/{mod_info.json, versions/{latest, 1.0.0}}`
/// Then within each version the actual files are placed
#[derive(Debug, Clone)]
pub struct ModCache {
    /// Full mod list from thunderstore
    thunderstore_mod_list: ModList,
    /// List of mods in the cache
    pub cache_mod_list: Vec<Mod>,
}
impl ModCache {
    pub fn new(mod_list: &ModList) -> Self {
        let mut cache = ModCache {
            thunderstore_mod_list: mod_list.clone(),
            cache_mod_list: vec![],
        };
        if let Err(e) = cache.update_self_from_cache() {
            eprintln!(
                "mod‑cache incomplete: {}",
                color_eyre::eyre::Report::from(e)
            );
        }
        cache
    }
    /// Adds a mod into the cache using a mod's ID. Will download from Thunderstore
    #[async_recursion]
    pub async fn cache_mod_by_mod_id(
        &mut self,
        id: &String,
        version: Option<&String>,
    ) -> Result<Mod> {
        let real_version = self.resolve_mod_version(id, version)?;
        let config = Config::new();
        let _mod_options = LocalModOptions::new(&config);
        let download_path = &config.mod_cache_directory;
        let this_mod = self
            .thunderstore_mod_list
            .mods
            .iter()
            .find(|x| x.uuid.to_string() == *id)
            .ok_or_else(|| eyre!("could not convert UUID to string"))?
            .clone();
        // check if mod is already cached
        if self.is_mod_in_cache(&this_mod.uuid, Some(&real_version)) {
            println!("{} already added to cache, skipping!", this_mod.name);
            return Ok(this_mod);
        }
        println!("caching mod: {}", this_mod.name);
        let thunderstore_version = this_mod
            .versions
            .iter()
            .find(|x| x.version_number == real_version)
            .ok_or_else(|| eyre!("cache_mod_by_mod_id: could not find version"))?
            .clone(); //I LOVE CLONE(). I LOVE NOT DEALING WITH THE BORROW CHECKER WOOOOO
        // Build the destination directory: <mod_cache_directory>/<mod id>/versions/<version id>
        let mod_id_folder = id;
        let version_id_folder = &thunderstore_version.version_number;
        let destination_dir = download_path
            .join(mod_id_folder)
            .join("versions")
            .join(version_id_folder);
        // Create the destination directory if it does not exist.
        tokio::fs::create_dir_all(&destination_dir).await?;
        let download_url = &thunderstore_version.download_url;
        let file_name = "extractme.zip";
        let destination_file = destination_dir.join(file_name);
        // download the mod file
        let response = reqwest::get(download_url).await?;
        if !response.status().is_success() {
            return Err(eyre!("Failed to download mod from {}", download_url));
        }
        let bytes = response.bytes().await?;
        // Save the downloaded bytes to the destination file.
        println!("destination file: {:?}", &destination_file);
        tokio::fs::write(&destination_file, &bytes).await?;
        // Extract the zip file contents
        //println!("Extracting mod files...");
        self.extract_zip_file(&destination_file, &destination_dir)
            .await?;
        // Optionally remove the zip file after extraction
        tokio::fs::remove_file(&destination_file).await?;
        // add the config.json to the file as well
        ModCache::add_mod_config_json(&this_mod, &config)?;

        // update config with the new version
        let mut local_mod_option = LocalModOptions::new(&config);
        local_mod_option.enable_mod(&this_mod, &config)?;
        println!("Hello! Just saying hi");
        self.cache_mod_dependancies(&this_mod, real_version).await?;

        self.update_self_from_cache()?;
        Ok(this_mod.clone())
    }

    async fn cache_mod_dependancies(&mut self, mod_to_cache: &Mod, version: String) -> Result<()> {
        let dependancies: Vec<Uuid> = {
            let dependancies_ref = self.get_mod_dependencies(mod_to_cache, Some(&version))?;
            dependancies_ref.iter().map(|x| x.uuid.clone()).collect()
        };
        println!("dependancies: {:?}", dependancies);
        for dependancy in dependancies {
            // just downloads the latest version for now, can always extract intended version from the mod's full-name later
            self.cache_mod_by_mod_id(&dependancy.to_string(), None)
                .await?;
        }
        Ok(())
    }

    /// Extract a zip file to the specified directory
    async fn extract_zip_file(&self, zip_path: &Path, extract_dir: &Path) -> Result<()> {
        // We need to use a blocking operation within a tokio thread since zip operations are synchronous
        let zip_path = zip_path.to_path_buf();
        let extract_dir = extract_dir.to_path_buf();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let file = std::fs::File::open(&zip_path)?;
            let mut archive = zip::ZipArchive::new(file)?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let outpath = match file.enclosed_name() {
                    Some(path) => extract_dir.join(path),
                    None => continue,
                };

                if file.name().ends_with('/') {
                    std::fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p)?;
                        }
                    }
                    let mut outfile = std::fs::File::create(&outpath)?;
                    std::io::copy(&mut file, &mut outfile)?;
                }

                // Get and set permissions
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = file.unix_mode() {
                        std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
                    }
                }
            }

            Ok(())
        })
        .await??; // Unwrap the JoinHandle and then the Result

        println!("Extraction completed successfully");
        Ok(())
    }

    fn add_mod_config_json(mod_to_save: &Mod, config: &Config) -> Result<()> {
        // Define the base cache directory.
        let cache_dir = Path::new(&config.mod_cache_directory);

        // Create the mod's directory using its id.
        let mod_dir = cache_dir.join(&mod_to_save.uuid.to_string());
        println!(
            "Adding config.json for mod: {} with ID: {}",
            mod_to_save.name, mod_to_save.uuid
        );
        fs::create_dir_all(&mod_dir)?;
        let mod_info_path = mod_dir.join("mod_info.json");
        // Serialize the mod object to a pretty JSON string.
        let json_content = serde_json::to_string_pretty(mod_to_save)?;
        // Write the serialized JSON to mod_info.json.
        // If the file already exists, it will be overwritten.
        fs::write(mod_info_path, json_content)?;

        Ok(())
    }

    /// Takes in a mod ID and optional version, and returns either the input version or the most recent version
    fn resolve_mod_version(&self, mod_id: &String, version: Option<&String>) -> Result<String> {
        // find the mod
        let this_mod = self
            .thunderstore_mod_list
            .mods
            .iter()
            .find(|x| x.uuid.to_string() == *mod_id)
            .ok_or(eyre!(
                "resolve_mod_version was passed a mod id that does not exist: {}",
                mod_id
            ))?;
        // Get the version
        match version {
            // If version is specified, verify it exists for this mod
            Some(version_to_find) => {
                let version_exists = this_mod
                    .versions
                    .iter()
                    .any(|v| v.version_number == *version_to_find);
                if !version_exists {
                    return Err(eyre!(
                        "Version {} not found for mod {}",
                        version_to_find,
                        this_mod.name
                    ));
                }
                Ok(version_to_find.clone())
            }
            // No version specified, return the latest version
            None => this_mod
                .versions
                .first()
                .ok_or(eyre!("Mod ({}) does not have any versions!", this_mod.name))
                .map(|v| v.version_number.clone()),
        }
    }
    /// Adds a mod to the Rumble mods folder
    pub async fn add_mod_to_rumble_by_id(
        &self,
        id: &String,
        version: Option<&String>,
    ) -> Result<Mod> {
        let config = Config::new();
        let rumble_mod_directory = config.rumble_directory.join("Mods");
        let rumble_user_data_directory = config.rumble_directory.join("UserData");
        let real_version = self.resolve_mod_version(id, version)?;
        let cache_directory = config.mod_cache_directory.join(id).join(real_version);

        let cache_mods = cache_directory
            .join("Mods")
            .read_dir()?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;
        for file in cache_mods {
            // for each file in the cache Mods folder
            fs::copy(file, &rumble_mod_directory)?; // copy mod to rumble folder
        }
        let cache_user_data = cache_directory
            .join("UserData")
            .read_dir()?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;
        for file in cache_user_data {
            let file_name = file.file_name().unwrap();
            let destination = rumble_user_data_directory.join(file_name);

            // Only copy if destination doesn't exist
            if !destination.exists() {
                if file.is_dir() {
                    fs::create_dir_all(&destination)?;
                    // If you need to copy directory contents recursively,
                    // you'd need to implement that logic here
                } else {
                    fs::copy(&file, &destination)?;
                }
            }
        }

        todo!()
    }
    /// Allows for reverse-searching. When given a path to a mod in the rumble folder, it will find where that mod is in the cache
    pub async fn get_mod_path_by_file_name(
        mod_file_name: String,
        version: String,
        config: Config,
    ) -> Result<PathBuf> {
        let search: Vec<String> = SearchBuilder::default()
            .location(config.mod_cache_directory)
            .search_input(mod_file_name)
            .depth(5)
            .build()
            .collect();
        // collected each file by the name, now check mod_info to see which version to use

        todo!()
    }
    /// Updates the in‑memory cache from the on‑disk cache directory.
    pub fn update_self_from_cache(&mut self) -> Result<Vec<color_eyre::eyre::Report>> {
        // ── 1. Locate / create the directory ──────────────────────────────
        let config = Config::new();
        let cache = config.mod_cache_directory.as_path();

        if !cache.exists() {
            fs::create_dir_all(cache)
                .wrap_err_with(|| format!("Failed to create mod‑cache directory at {:?}", cache))?;
        }

        let dir = cache
            .read_dir()
            .wrap_err_with(|| format!("Could not read directory {:?}", cache))?;

        // ── 2. Scan directory; collect mods, keep individual errors ───────
        let mut errors = Vec::new();

        let mods: Vec<Mod> = dir
            .filter_map(|entry| entry.ok()) // ignore IO errors here
            .filter_map(|entry| {
                match ModCache::get_mod_from_dir_in_cache(&entry.path()) {
                    Ok(m) => Some(Ok(m)),
                    Err(e) => {
                        errors.push(e); // non‑fatal: remember and keep going
                        None
                    }
                }
            })
            .collect::<Result<Vec<_>>>()?; // stops if *this* map yields an Err

        // ── 3. Update self and return ────────────────────────────────────
        self.cache_mod_list = mods;
        Ok(errors) // empty vec ⇒ no non‑fatal errors
    }

    /// returns a full mod object if given a path like: `[Cache Dir]/[Mod ID]`
    /// WARNING: returns full mod list from thunderstore - use `update_versions_in_mod` if you want the mod's versions to match the cache
    pub fn get_mod_from_dir_in_cache(path: &Path) -> Result<Mod> {
        if !path.is_dir() {
            return Err(eyre!(
                "Cannot get mod from a nonexistant directory! Failed with path: {:?}",
                path
            ));
        }
        let mod_info_path = path.join("mod_info.json");
        if mod_info_path.exists() {
            let content = fs::read_to_string(mod_info_path)?;
            let response: Mod = serde_json::from_str(&content)?;
            return Ok(response);
        }
        Err(eyre!("mod_info.json not found in mod path: {:?}", path))
    }
    /// Removes any versions in the mod that are not present in the cache
    pub fn update_versions_in_mod(&self, config: &Config, mod_to_update: &Mod) -> Result<Mod> {
        let version_path = self
            .get_mod_file_by_id(config, mod_to_update.uuid)?
            .join("versions");
        let mut file_version_names: Vec<String> = vec![];
        for dirent in fs::read_dir(&version_path)? {
            let name = dirent?
                .file_name()
                .into_string()
                .map_err(|bad: OsString| eyre!("file name is not valid UTF‑8: {:?}", bad))?; // <- converts the error
            file_version_names.push(name);
        }
        let updated_versions = mod_to_update
            .versions
            .iter()
            .filter(|x| file_version_names.contains(&x.version_number))
            .map(|x| x.clone()) // not good, but works
            .collect();
        let new_mod = Mod {
            versions: updated_versions,
            ..mod_to_update.clone()
        };
        Ok(new_mod)
    }
    /// returns the mods that are dependancies of this mod. Throws an error if a mod is listed as a dependency but not found within the thunderstore list.
    /// If the version is not given, uses the latest version
    pub fn get_mod_dependencies<'a>(
        &'a self,
        mod_to_get: &'a Mod,
        version_to_get: Option<&String>,
    ) -> Result<Vec<&'a Mod>> {
        let real_version =
            self.resolve_mod_version(&mod_to_get.uuid.to_string(), version_to_get)?;
        let version = mod_to_get
            .versions
            .iter()
            .find(|v| v.version_number == real_version)
            .ok_or_else(|| {
                eyre!(
                    "Version '{}' not found for '{}'",
                    real_version,
                    mod_to_get.name
                )
            })?;
        version
            .dependencies
            .iter()
            .map(|dep_name| {
                self.get_mod_from_full_mod_name(dep_name).ok_or_else(|| {
                    eyre!(
                        "Dependency '{}' declared by '{}' was not found in Thunderstore",
                        dep_name,
                        mod_to_get.name
                    )
                })
            })
            .collect()
    }
    pub fn get_mod_from_full_mod_name(&self, full_name: &String) -> Option<&Mod> {
        self.thunderstore_mod_list.mods.iter().find(|x| {
            x.versions
                .iter()
                .find(|y| y.full_name == *full_name)
                .is_some()
        })
    }
    // returns `[mod cache]/[mod id]`
    fn get_mod_file_by_id(&self, config: &Config, id: Uuid) -> Result<PathBuf> {
        let path = config.mod_cache_directory.join(id.to_string());
        if !path.exists() {
            return Err(eyre!("Could not find mod file in cache!"));
        }
        Ok(path)
    }
    fn get_mods_from_cache(&self) -> &Vec<Mod> {
        &self.cache_mod_list
    }
    // If no string is passed, will return true for any version. Otherwise, will only return true if that version is present in the cache
    pub fn is_mod_in_cache(&self, uuid: &Uuid, version: Option<&String>) -> bool {
        self.cache_mod_list
            .iter()
            .find(|m| m.uuid == *uuid)
            .map_or(false, |m| {
                self.update_versions_in_mod(&Config::new(), m)
                    .is_ok_and(|x| {
                        x.versions
                            .iter()
                            .any(|v| v.version_number == *version.unwrap())
                    })
            })
    }

    // Returns None if the mod is not in cache, returns Some(false) if the version is not in cache
    pub fn does_mod_have_version(&self, mod_to_check: &Mod, version: &String) -> Option<bool> {
        Some(
            mod_to_check
                .versions
                .iter()
                .any(|v| &v.version_number == version),
        )
    }
}
