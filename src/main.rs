use std::fmt::Display;

use color_eyre::eyre::{eyre, Result};

mod gui;
mod thunderstore;

use thunderstore::ModList;
use gui::start;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let list = ModList::new().await?;
    match start(list) {
        Ok(a) => Ok(()),
        Err(e) => Err(eyre!(e.to_string()))
    }
}