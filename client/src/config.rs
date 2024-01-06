use std::{fs, path::PathBuf, sync::OnceLock};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub host: String,
    pub token: String,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn load_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let config_path = config_path();

        let data = fs::read(&config_path)
            .expect(format!("Error reading config at {}", config_path.display()).as_str());

        serde_json::from_slice(&data).unwrap()
    })
}

#[cfg(target_os = "linux")]
fn config_path() -> PathBuf {
    use std::env;

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

#[cfg(target_os = "windows")]
fn config_path() -> PathBuf {
    use windows::Win32::{
        Foundation::{HANDLE, HWND},
        UI::Shell::{SHGetFolderPathW, CSIDL_APPDATA, SHGFP_TYPE_CURRENT},
    };

    let mut raw_path = [0u16; 260];
    unsafe {
        SHGetFolderPathW(
            HWND::default(),
            CSIDL_APPDATA as i32,
            HANDLE::default(),
            SHGFP_TYPE_CURRENT.0 as u32,
            &mut raw_path,
        )
        .expect("Unable to get default config path");
    }
    let path_str = String::from_utf16(&raw_path).unwrap();
    let config_folder_path = PathBuf::from(path_str.trim_matches(char::from(0)));

    let error_msg = format!(
        "Config folder doesnt exist ({})",
        config_folder_path.display()
    );
    if !config_folder_path.try_exists().expect(&error_msg) {
        panic!("{}", error_msg);
    }

    config_folder_path.join("shortclip-config.json")
}
