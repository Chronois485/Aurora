pub mod executor;
pub mod parser;

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
