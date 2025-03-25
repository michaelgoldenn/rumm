use std::{fs::File, io::Write};

use serde::Deserialize;
use color_eyre::eyre::{eyre, Result};

#[derive(Deserialize, Clone, Debug)]
pub struct Mod {
    #[serde(rename(deserialize = "uuid4"))]
    pub id: String,
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
#[derive(Deserialize, Clone, Debug)]
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
    pub website_url: String
}
#[derive(Clone, Debug)]
pub struct ModList {
    pub mods: Vec<Mod>,
}


impl Mod {
}

impl ModList {
    pub async fn new() -> Result<Self> {
        let request_url = format!("https://thunderstore.io/c/rumble/api/v1/package/");
        
        // get the response
        let response_text = reqwest::get(&request_url)
            .await?
            .text()
            .await?;
        
        // debug: print json to console
        //println!("Raw JSON Response:\n{}", 
        //    serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&response_text)?)?
        //);
        
        let response: Vec<Mod> = serde_json::from_str(&response_text)?;
        
        return Ok(Self { 
            mods: response,
        });
    }
}