mod app;
mod test_cases;
mod types;
mod utils;

use anyhow::{Ok, Result};

use crate::app::cli;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    cli().await?;
    Ok(())
}
