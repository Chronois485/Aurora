pub mod executor {
    use crate::Command;
    use std::process::Command as SysCommand;

    pub fn execute(cmd: Command) -> bool {
        match cmd {
            Command::OpenFirefox => {
                let _ = SysCommand::new("firefox").spawn();
                true
            }
            Command::OpenTerminal => {
                let _ = SysCommand::new("ghostty").spawn();
                true
            }
            Command::VolumeUp => {
                let _ = SysCommand::new("wpctl")
                    .args(["set-volume", "@DEFAULT_AUDIO_SINK@", "5%+"])
                    .spawn();
                true
            }
            Command::VolumeDown => {
                let _ = SysCommand::new("wpctl")
                    .args(["set-volume", "@DEFAULT_AUDIO_SINK@", "5%-"])
                    .spawn();
                true
            }
            Command::Quit => {
                println!("Bye bye");
                false
            }
            Command::Unknown(t) => {
                println!("Unknown command: {}", t);
                true
            }
        }
    }
}
