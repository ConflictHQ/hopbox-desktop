use crate::config::Config;
use anyhow::Result;

pub struct App {
    config: Config,
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn run(&mut self) -> Result<()> {
        // TODO: initialize TUI, start PTY session, enter event loop
        // Milestone v0: implement terminal rendering + AI copilot loop
        println!("hopbox starting... (TUI not yet implemented)");
        Ok(())
    }
}
