use colored::Colorize;
use serde_json::{Map, Value};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

type JsonMap = Map<String, Value>;

const DEFAULT_SETTINGS: &str = r#"{
    "text_mode": "false",
    "conversation_mode": "false",
    "language": "uk",
    "model": "normal",
    "fuzzy_matcher_threshold": "0.85"
}"#;

pub struct SettingsManager {
    file_path: String,
    settings: JsonMap,
}

impl SettingsManager {
    fn create_default_settings(file_path: &str) {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .expect("Unable to create settings file");

        file.write_all(DEFAULT_SETTINGS.as_bytes())
            .expect("Unable to write default settings");
    }

    fn read_settings(&mut self) {
        let path = Path::new(&self.file_path);

        if !path.exists() {
            Self::create_default_settings(&self.file_path);
        }

        let mut file = File::open(&self.file_path).expect("Unable to open the file");
        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .expect("Unable to read the file");

        self.settings = serde_json::from_str(&contents).unwrap();
    }

    pub fn new(file_path: String) -> Self {
        let mut manager = Self {
            file_path,
            settings: JsonMap::new(),
        };
        manager.read_settings();
        manager
    }

    pub fn get_setting(&self, setting: &str) -> String {
        if let Some(s) = self.settings.get(setting) {
            s.as_str().unwrap_or("").to_string()
        } else {
            String::from("")
        }
    }
    pub fn print_settings(&self) {
        println!("{}", "[*] Settings".bold().magenta());
        for setting in self.settings.keys() {
            println!(
                "{}",
                format!("    - {}: {}", setting, self.settings[setting]).magenta()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::SETTINGS_FILE_PATH;

    use super::SettingsManager;

    #[test]
    fn test_get_setting_success() {
        let settings_manager = SettingsManager::new(String::from(SETTINGS_FILE_PATH));
        let text_mode = settings_manager.get_setting("text_mode");
        assert_ne!(text_mode, "");
    }

    #[test]
    fn test_get_setting_invalid() {
        let settings_manager = SettingsManager::new(String::from(SETTINGS_FILE_PATH));
        let invalid_settings = settings_manager.get_setting("invalid_settings");
        assert_eq!(invalid_settings, String::from(""));
    }
}
