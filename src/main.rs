mod app;
mod ai;
mod config;
mod session;
mod tui;

use anyhow::Result;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::load()?;
    let mut app = app::App::new(config)?;
    app.run().await?;
    Ok(())
}
