use colored::Colorize;
use std::{collections::HashMap, fs::File, io::Read};

type JsonMap = HashMap<String, String>;

fn read_settings(file_path: &str) -> JsonMap {
    let mut file = File::open(file_path).expect("Unable to open the file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Unable to read the file");

    let data: JsonMap = serde_json::from_str(&contents).unwrap();

    data
}

pub fn get_setting(setting: &str, file_path: &str) -> String {
    let settings = read_settings(file_path);
    if let Some(s) = settings.get(setting) {
        s.clone().to_lowercase()
    } else {
        String::from("")
    }
}

pub fn print_settings(file_path: &str) {
    let settings = read_settings(file_path);
    println!("{}", "[*] Settings".bold().magenta());
    for setting in settings.keys() {
        println!(
            "{}",
            format!("    - {}: {}", setting, settings[setting]).magenta()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_setting_success() {
        let text_mode = get_setting("text_mode", "settings.json");
        assert_ne!(text_mode, "");
    }

    #[test]
    fn test_get_setting_invalid() {
        let invalid_settings = get_setting("invalid_settings", "settings.json");
        assert_eq!(invalid_settings, String::from(""));
    }
}
