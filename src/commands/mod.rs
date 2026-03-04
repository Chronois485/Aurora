pub mod executor;
pub mod parser;

use crate::{settings::manager::SettingsManager, SETTINGS_FILE_PATH};
use strsim::jaro_winkler;

#[derive(Debug, Clone)]
pub enum SystemToggles {
    Volume,
    Wifi,
    Bluetooth,
    NightLight,
    DoNotDisturb,
}

impl SystemToggles {
    pub fn _iter() -> impl Iterator<Item = Self> {
        [
            SystemToggles::Volume,
            SystemToggles::Wifi,
            SystemToggles::Bluetooth,
            SystemToggles::NightLight,
            SystemToggles::DoNotDisturb,
        ]
        .into_iter()
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    OpenApp(App),
    OpenFolder(String),
    VolumeUp,
    VolumeDown,
    VolumeMax,
    BrightnessUp,
    BrightnessDown,
    BrightnessMax,
    BrightnessMin,
    AudioPause,
    AudioNext,
    AudioPrevious,
    Poweroff,
    Reboot,
    Sleep,
    FindInInternet(String),
    EndConversation,
    Screenshot,
    SystemToggle(SystemToggles),
    Quit,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub enum App {
    Firefox,
    Terminal,
    Obsidian,
    Telegram,
    Steam,
    Dolphin,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    Running,
    EndConversation,
    Quit,
}

fn has_any(text: &str, needles: &[&str]) -> bool {
    let settings_manager = SettingsManager::new(String::from(SETTINGS_FILE_PATH));
    let treshold = settings_manager.get_setting("fuzzy_matcher_threshold");

    let treshold: f64 = treshold.parse().unwrap_or(0.85);

    needles.iter().any(|needle| {
        if text.contains(*needle) {
            return true;
        }

        text.split_ascii_whitespace()
            .any(|word| jaro_winkler(word, needle) >= treshold)
    })
}
