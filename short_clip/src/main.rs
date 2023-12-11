use std::sync::OnceLock;

mod config;
mod upload;

use config::{load_config, Config};
use upload::handle_hotkey;

mod hotkey;

pub static CONFIG: OnceLock<Config> = OnceLock::new();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    CONFIG.set(load_config()).unwrap();

    hotkey::create_listener(handle_hotkey)
}
