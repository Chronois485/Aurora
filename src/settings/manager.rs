use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

type JsonMap = HashMap<String, String>;

pub fn read_settings(file_path: &str) -> JsonMap {
    let mut file = File::open(file_path).expect("Unable to open the file");
    let mut contents = String::new();

    file.read_to_string(&mut contents).expect("Unable to read the file");

    let data: JsonMap = serde_json::from_str(&contents).unwrap();

    data
}

#[test]
fn test_read_settings() {
    let data = read_settings("settings.json");
    assert_eq!(data["text_mode"], "true");
}