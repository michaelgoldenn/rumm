use color_eyre::eyre::Result;
use self_update::{cargo_crate_version, update::Release};

/// Allows the program to update itself from the latest github release
pub struct Updater {
    releases: Vec<Release>,
    current_release_version: String,
    repo_owner: String,
    repo_name: String,
}

impl Updater {
    pub async fn new() -> Result<Self> {
        let repo_owner = "michaelgoldenn".to_string();
        let repo_name = "rumm".to_string();
        let releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner(&repo_owner)
            .repo_name(&repo_name)
            .build()?
            .fetch()?;
        Ok(Updater {
            releases,
            current_release_version: cargo_crate_version!().to_string(),
            repo_owner,
            repo_name,
        })
    }
    pub fn get_release_names(&self) -> Vec<String> {
        self.releases.iter().map(|x| x.name.clone()).collect()
    }
    pub fn get_release_versions(&self) -> Vec<String> {
        self.releases.iter().map(|x| x.version.clone()).collect()
    }
    pub fn get_releases(&self) -> &Vec<Release> {
        &self.releases
    }
    pub fn get_releases_mut(&mut self) -> &mut Vec<Release> {
        &mut self.releases
    }
    pub fn sync_releases_with_remote(&mut self) -> Result<()> {
        self.releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner(&self.repo_owner)
            .repo_name(&self.repo_name)
            .build()?
            .fetch()?;
        Ok(())
    }
    pub fn is_mod_up_to_date(&self) -> bool {
        self.releases.first().map_or(false, |x| x.version == self.current_release_version)
    }
}
