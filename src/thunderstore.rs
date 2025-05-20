use std::path::PathBuf;

use color_eyre::eyre::{Ok, Result};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config_and_such::SortType;

/// Just the straight mod data deserialized from Thunderstore's API request
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Mod {
    #[serde(rename(deserialize = "uuid4", serialize = "uuid4"))]
    pub uuid: Uuid,
    pub name: String,
    pub full_name: String,
    pub owner: String,
    pub package_url: String,
    pub donation_link: Option<String>,
    pub date_created: String,
    pub date_updated: String,
    pub rating_score: i32,
    pub is_pinned: bool,
    pub is_deprecated: bool,
    pub has_nsfw_content: bool,
    pub categories: Vec<String>,
    pub versions: Vec<Version>,
}
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Version {
    pub date_created: String,
    pub dependencies: Vec<String>,
    pub description: String,
    pub download_url: String,
    pub downloads: i32,
    pub file_size: i32,
    pub full_name: String,
    pub icon: String,
    pub is_active: bool,
    pub name: String,
    pub uuid4: String,
    pub version_number: String,
    pub website_url: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModList {
    pub mods: Vec<Mod>,
}

impl Mod {
    /// updates the versions of the mod from thunderstore.
    /// returns `true` if the verisons have changed, `false` otherwise
    pub async fn update(&mut self) -> Result<bool> {
        // the docs say that this url format is depricated, but I didn't see a better way to do this
        let request_url = format!(
            "https://thunderstore.io/c/rumble/api/v1/package/{}",
            self.uuid
        );
        let response_text = reqwest::get(&request_url).await?.text().await?;
        let response: Mod = serde_json::from_str(&response_text)?;
        let mut have_versions_changed: bool = false;
        if self.versions != response.versions {
            have_versions_changed = true
        }
        self.versions = response.versions;
        Ok(have_versions_changed)
    }
    pub async fn new(url: Url) -> Result<Self> {
        let response = reqwest::get(url).await?.text().await?;
        let parsed: Mod = serde_json::from_str(&response)?;
        Ok(parsed)
    }
}
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    #[tokio::test]
    async fn test() -> Result<()> {
        Mod::new(Url::from_str("https://thunderstore.io/c/rumble/api/v1/package/").unwrap())
            .await
            .unwrap();
        Ok(())
    }
}

impl ModList {
    /// If nothing is found at the path, just makes a new empty cache
    pub fn new(path: PathBuf) -> Result<Self> {
        // return the cached thunderstore response
        let file = std::fs::read_to_string(path);
        match file {
            std::result::Result::Ok(x) => Ok(serde_json::from_str::<Self>(x.as_str())?),
            Err(_) => Ok(ModList { mods: vec![] }),
        }
    }

    pub fn sort(&self, metric: &SortType) -> ModList {
        let mut new_list = self.clone();
        new_list.sort_self(metric);
        new_list
    }

    pub fn sort_self(&mut self, metric: &SortType) {
        match metric {
            SortType::Alphabetically => {
                self.mods.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SortType::ReleaseDate => {
                self.mods.sort_by(|a, b| {
                    b.versions.last().unwrap().date_created.cmp(&a.versions.last().unwrap().date_created)
                });
            }
            SortType::UpdateDate => {
                self.mods.sort_by(|a, b| {
                    b.versions.first().unwrap().date_created.cmp(&a.versions.first().unwrap().date_created)
                });
            }
        }
    }
}
