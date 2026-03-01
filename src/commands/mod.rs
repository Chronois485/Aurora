pub mod executor;
pub mod parser;

#[derive(Debug, Clone)]
pub enum Command {
    OpenApp(App),
    VolumeUp,
    VolumeDown,
    VolumeMute,
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
