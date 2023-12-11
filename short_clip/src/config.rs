use std::{env, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub host: String,
    pub token: String,
}

pub fn load_config() -> Config {
    let config_path = config_path();

    let data = fs::read(&config_path)
        .expect(format!("Error reading config at {}", config_path.display()).as_str());

    serde_json::from_slice(&data).unwrap()
}

fn config_path() -> PathBuf {
    let home = PathBuf::from(env::var("HOME").expect("No HOME set"));
    let mut config_folder_path = home.join(".config");

    if let Ok(xdg_home) = env::var("XDG_CONFIG_HOME") {
        config_folder_path = PathBuf::from(xdg_home);
    }

    let error_msg = format!(
        "Config folder doesnt exist ({})",
        config_folder_path.display()
    );
    if !config_folder_path.try_exists().expect(&error_msg) {
        panic!("{}", error_msg);
    }

    config_folder_path.join("shortclip-config.json")
}
