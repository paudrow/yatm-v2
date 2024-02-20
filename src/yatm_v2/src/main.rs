mod app;
mod constants;
mod helpers;
mod types;

use anyhow::{Ok, Result};

use crate::app::cli;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    cli().await?;
    Ok(())
}
