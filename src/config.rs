use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};

use crate::FuzzierTheme;

const DEFAULT_CONFIG: &str = r#"
    "theme":"Dracula"
"#;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub theme: FuzzierTheme,
}

impl Config {
    pub fn default() -> Config {
        Config {
            theme: FuzzierTheme::Dracula,
        }
    }
}

pub fn get_config() -> Config {
    let config_path = "preferences.json";

    match std::fs::read_to_string(config_path) {
        Ok(content) => match from_str::<Config>(content.as_str()) {
            Ok(preferences) => return preferences,
            Err(_) => return Config::default(),
        },
        Err(_) => return Config::default(),
    };
}

pub fn update_theme(theme: &FuzzierTheme) {
    let mut config_dir = dirs::config_dir().unwrap();
    config_dir.push("fuzzier");

    let config_path = config_dir.join("config.json");

    let json = match std::fs::read_to_string(&config_path) {
        Ok(json) => json,
        Err(_) => DEFAULT_CONFIG.to_string(),
    };

    let mut config = from_str::<Config>(json.as_str()).unwrap_or(Config {
        theme: FuzzierTheme::Dracula,
    });

    config.theme = theme.clone();

    let serialized = to_string_pretty(&config).unwrap_or(DEFAULT_CONFIG.to_string());

    let _ = std::fs::write(config_path, serialized);
}
