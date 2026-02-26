pub mod executor;
pub mod parser;

#[derive(Debug, Clone)]
pub enum Command {
    OpenApp(App),
    VolumeUp,
    VolumeDown,
    AudioPause,
    AudioNext,
    AudioPrevious,
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
