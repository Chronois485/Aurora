use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

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
        s.clone()
    } else {
        String::from("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_settings() {
        let data = read_settings("settings.json");
        assert_eq!(data["text_mode"], "true");
    }

    #[test]
    fn test_get_setting_success() {
        let text_mode = get_setting("text_mode", "settings.json");
        assert_eq!(text_mode, "true");
    }

    #[test]
    fn test_get_setting_invalid() {
        let invalid_settings = get_setting("invalid_settings", "settings.json");
        assert_eq!(invalid_settings, String::from(""));
    }
}
