use crate::{
    commands::{CommandResult, SystemToggles},
    settings::manager::SettingsManager,
    SETTINGS_FILE_PATH,
};

use super::{has_any, App, Command};

pub trait Runner {
    fn spawn(&mut self, program: &str, args: &[&str]) -> bool;
    fn exec_output(&mut self, program: &str, args: &[&str]) -> Option<String>;
}

pub struct SystemRunner;

impl Runner for SystemRunner {
    fn spawn(&mut self, program: &str, args: &[&str]) -> bool {
        std::process::Command::new(program)
            .args(args)
            .spawn()
            .is_ok()
    }

    fn exec_output(&mut self, program: &str, args: &[&str]) -> Option<String> {
        std::process::Command::new(program)
            .args(args)
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
    }
}

pub fn execute_with<R: Runner>(runner: &mut R, cmd: Command) -> CommandResult {
    match cmd {
        Command::SwitchWorkspace(workspace) => {
            switch_workspace(runner, workspace);
            CommandResult::Running
        }
        Command::OpenApp(app) => {
            open_app(runner, app);
            CommandResult::Running
        }
        Command::VolumeUp => {
            set_volume(runner, "5%+");
            CommandResult::Running
        }
        Command::VolumeDown => {
            set_volume(runner, "5%-");
            CommandResult::Running
        }
        Command::AudioPause => {
            audio_pause(runner);
            CommandResult::Running
        }
        Command::AudioNext => {
            audio_next(runner);
            CommandResult::Running
        }
        Command::AudioPrevious => {
            audio_previous(runner);
            CommandResult::Running
        }
        Command::FindInInternet(prompt) => {
            find_in_internet(runner, &prompt);
            CommandResult::Running
        }
        Command::EndConversation => CommandResult::EndConversation,
        Command::Screenshot => {
            screenshot(runner);
            CommandResult::Running
        }
        Command::BrightnessDown => {
            set_brightness(runner, "10%-");
            CommandResult::Running
        }
        Command::BrightnessUp => {
            set_brightness(runner, "10%+");
            CommandResult::Running
        }
        Command::BrightnessMax => {
            set_brightness(runner, "100%");
            CommandResult::Running
        }
        Command::BrightnessMin => {
            set_brightness(runner, "5%");
            CommandResult::Running
        }
        Command::VolumeMax => {
            set_volume(runner, "100%");
            CommandResult::Running
        }
        Command::SystemToggle(toggle) => {
            system_toggle(runner, toggle);
            CommandResult::Running
        }
        Command::Poweroff => {
            poweroff(runner);
            CommandResult::Quit
        }
        Command::Reboot => {
            reboot(runner);
            CommandResult::Quit
        }
        Command::Sleep => {
            sleep(runner);
            CommandResult::Running
        }
        Command::OpenFolder(folder) => {
            let settings_manager = SettingsManager::new(String::from(SETTINGS_FILE_PATH));
            let quick_folders = settings_manager.get_setting("quick_folders");
            let folder_lower = folder.to_lowercase();
            for quick_folder in quick_folders.split(';') {
                let parts: Vec<&str> = quick_folder.split(':').collect();
                if parts.len() == 2 {
                    let (destination, key_words) = (parts[0], parts[1]);
                    let key_words: Vec<String> =
                        key_words.split(',').map(|s| s.to_lowercase()).collect();
                    let key_words: Vec<&str> = key_words.iter().map(|s| s.as_str()).collect();
                    if has_any(&folder_lower, &key_words) {
                        println!("{}\n{:?}\n{}", folder_lower, key_words, destination);
                        if settings_manager.get_setting("open_folder_in_terminal") == "true" {
                            runner.spawn("kitty", &["-d", destination]);
                        } else {
                            runner.spawn("dolphin", &[destination]);
                        }
                        return CommandResult::Running;
                    }
                } else {
                    continue;
                }
            }
            CommandResult::Running
        }
        Command::Quit => CommandResult::Quit,
        Command::Unknown(_text) => CommandResult::Running,
    }
}

fn switch_workspace<R: Runner>(runner: &mut R, workspace: u8) {
    match runner.exec_output("sh", &["-c", "echo $XDG_CURRENT_DESKTOP"]) {
        Some(val) => {
            if val.contains("Hyprland") {
                runner.spawn(
                    "~/.config/hypr/hyprland/scripts/workspace_action.sh",
                    &["workspace", &workspace.to_string()],
                );
                println!("hyprland workspace: {}", &workspace.to_string());
            } else if val.contains("KDE") {
                runner.spawn(
                    "qdbus6",
                    &[
                        "org.kde.KWin",
                        "/KWin",
                        "setCurrentDesktop",
                        &workspace.to_string(),
                    ],
                );
            }
        }
        None => {}
    }
}

fn open_app<R: Runner>(runner: &mut R, app: App) {
    match app {
        App::Firefox => {
            runner.spawn("firefox", &[]);
        }
        App::Terminal => {
            runner.spawn("kitty", &[]);
        }
        App::Dolphin => {
            runner.spawn("dolphin", &[]);
        }
        App::Obsidian => {
            if !runner.spawn("obsidian", &[]) {
                runner.spawn("flatpak", &["run", "md.obsidian.Obsidian"]);
            }
        }
        App::Steam => {
            if !runner.spawn("steam", &[]) {
                runner.spawn("flatpak", &["run", "com.valvesoftware.Steam"]);
            }
        }
        App::Telegram => {
            if !runner.spawn("Telegram", &[]) && !runner.spawn("telegram-desktop", &[]) {
                runner.spawn("flatpak", &["run", "org.telegram.desktop"]);
            }
        }
    }
}

fn system_toggle<R: Runner>(runner: &mut R, toggle: SystemToggles) {
    match toggle {
        SystemToggles::Volume => {
            runner.spawn("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]);
        }
        SystemToggles::Wifi => {
            toggle_wifi(runner);
        }
        SystemToggles::Bluetooth => {
            toggle_bluetooth(runner);
        }
        SystemToggles::NightLight => {
            toggle_night_light(runner);
        }
        SystemToggles::DoNotDisturb => {
            run_kde_command(runner, "/component/plasmashell", "toggle do not disturb");
        }
    }
}

fn toggle_wifi<R: Runner>(runner: &mut R) {
    let wifi_status = runner.exec_output("nmcli", &["-t", "-f", "wifi", "radio"]);

    if wifi_status.as_ref().map(|s| s.trim()) == Some("enabled") {
        runner.spawn("nmcli", &["radio", "wifi", "off"]);
    } else {
        runner.spawn("nmcli", &["radio", "wifi", "on"]);
    }
}

fn toggle_night_light<R: Runner>(runner: &mut R) {
    let night_light_status = runner.exec_output("xsct", &[]);
    if let Some(status) = night_light_status.as_ref() {
        let night_light_status: u16 = match status
            .chars()
            .filter(|char| char.is_digit(10))
            .take(5)
            .collect::<String>()
            .parse()
        {
            Ok(status) => status,
            Err(_) => 1000,
        };
        if night_light_status == 6500 {
            runner.spawn("xsct", &["4500"]);
        } else {
            runner.spawn("xsct", &["6500"]);
        }
    }
}

fn toggle_bluetooth<R: Runner>(runner: &mut R) {
    let bluetooth_status = runner.exec_output("bluetooth", &[]);
    if bluetooth_status.as_ref().map(|s| s.trim()) == Some("bluetooth = on") {
        runner.spawn("bluetoothctl", &["power", "off"]);
    } else {
        runner.spawn("bluetoothctl", &["power", "on"]);
    }
}

fn run_kde_command<R: Runner>(runner: &mut R, component: &str, program: &str) {
    runner.spawn(
        "qdbus6",
        &[
            "org.kde.kglobalaccel",
            component,
            "org.kde.kglobalaccel.Component.invokeShortcut",
            format!("\"{}\"", program).as_str(),
        ],
    );
}

fn poweroff<R: Runner>(runner: &mut R) {
    runner.spawn("poweroff", &[]);
}

fn reboot<R: Runner>(runner: &mut R) {
    runner.spawn("reboot", &[]);
}

fn sleep<R: Runner>(runner: &mut R) {
    runner.spawn("systemctl", &["suspend"]);
}

fn find_in_internet<R: Runner>(runner: &mut R, prompt: &String) {
    runner.spawn(
        "xdg-open",
        &[format!("https://www.google.com/search?q={}", prompt).as_str()],
    );
}

fn screenshot<R: Runner>(runner: &mut R) {
    runner.spawn("spectacle", &[]);
}

fn set_volume<R: Runner>(runner: &mut R, delta: &str) {
    runner.spawn("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", delta]);
}

fn set_brightness<R: Runner>(runner: &mut R, delta: &str) {
    runner.spawn("brightnessctl", &["set", delta]);
}

fn audio_pause<R: Runner>(runner: &mut R) {
    runner.spawn("playerctl", &["play-pause"]);
}
fn audio_next<R: Runner>(runner: &mut R) {
    runner.spawn("playerctl", &["next"]);
}

fn audio_previous<R: Runner>(runner: &mut R) {
    // Call twice to skip to the previous track (first call restarts current track)
    runner.spawn("playerctl", &["previous"]);
    runner.spawn("playerctl", &["previous"]);
}

pub fn execute(cmd: Command) -> CommandResult {
    let mut r = SystemRunner;
    execute_with(&mut r, cmd)
}

#[cfg(test)]
mod tests {
    use rand::random;

    use super::*;
    use crate::commands::{App, Command};

    #[derive(Default)]
    struct FakeRunner {
        calls: Vec<(String, Vec<String>)>,
        fail_obsidian: bool,
        fail_telegram: bool,
        fail_telegram_desktop: bool,
        fail_steam: bool,
        exec_output_values: std::collections::HashMap<String, String>,
        enviroment: String,
    }

    impl Runner for FakeRunner {
        fn spawn(&mut self, program: &str, args: &[&str]) -> bool {
            self.calls.push((
                program.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ));

            if self.fail_obsidian && program == "obsidian" {
                return false;
            }
            if self.fail_telegram && program == "Telegram" {
                return false;
            }
            if self.fail_telegram_desktop && program == "telegram-desktop" {
                return false;
            }
            if self.fail_steam && program == "steam" {
                return false;
            }

            true
        }

        fn exec_output(&mut self, program: &str, args: &[&str]) -> Option<String> {
            self.calls.push((
                program.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ));

            if program == "nmcli" && args == ["-t", "-f", "wifi", "radio"] {
                use rand::random;
                let wifi_enabled: bool = random();
                if wifi_enabled {
                    Some("enabled\n".to_string())
                } else {
                    Some("disabled\n".to_string())
                }
            } else if program == "sh" && args == ["-c", "echo $XDG_CURRENT_DESKTOP"] {
                Some(format!("{}\n", self.enviroment))
            } else if let Some(value) = self.exec_output_values.get(program) {
                Some(value.clone())
            } else {
                None
            }
        }
    }

    #[test]
    fn execute_open_firefox_spawns_firefox() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::OpenApp(App::Firefox));
        assert_eq!(keep, CommandResult::Running);
        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "firefox");
    }

    #[test]
    fn execute_open_dolphin_spawns_dolphin() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::OpenApp(App::Dolphin));
        assert_eq!(keep, CommandResult::Running);
        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "dolphin");
    }

    #[test]
    fn execute_open_telegram_spawns_telegram() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::OpenApp(App::Telegram));
        assert_eq!(keep, CommandResult::Running);
        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "Telegram");
    }

    #[test]
    fn telegram_fallbacks_to_telegram_desktop_when_direct_spawn_fails() {
        let mut r = FakeRunner {
            fail_telegram: true,
            ..Default::default()
        };
        let keep = execute_with(&mut r, Command::OpenApp(App::Telegram));
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 2);
        assert_eq!(r.calls[0].0, "Telegram");
        assert_eq!(r.calls[1].0, "telegram-desktop");
    }

    #[test]
    fn telegram_fallbacks_to_flatpak_when_desktop_spawn_fails() {
        let mut r = FakeRunner {
            fail_telegram: true,
            fail_telegram_desktop: true,
            ..Default::default()
        };
        let keep = execute_with(&mut r, Command::OpenApp(App::Telegram));
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 3);
        assert_eq!(r.calls[0].0, "Telegram");
        assert_eq!(r.calls[1].0, "telegram-desktop");
        assert_eq!(r.calls[2].0, "flatpak");
        assert_eq!(r.calls[2].1, vec!["run", "org.telegram.desktop"]);
    }

    #[test]
    fn execute_volume_up_calls_wpctl() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::VolumeUp);
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "wpctl");
        assert_eq!(
            r.calls[0].1,
            vec!["set-volume", "@DEFAULT_AUDIO_SINK@", "5%+"]
        );
    }

    #[test]
    fn execute_quit_stops() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::Quit);
        assert_eq!(keep, CommandResult::Quit);
        assert!(r.calls.is_empty());
    }

    #[test]
    fn execute_open_obsidian_spawns_obsidian() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::OpenApp(App::Obsidian));
        assert_eq!(keep, CommandResult::Running);
        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "obsidian");
    }

    #[test]
    fn obsidian_fallbacks_to_flatpak_when_direct_spawn_fails() {
        let mut r = FakeRunner {
            fail_obsidian: true,
            ..Default::default()
        };
        let keep = execute_with(&mut r, Command::OpenApp(App::Obsidian));
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 2);
        assert_eq!(r.calls[0].0, "obsidian");
        assert_eq!(r.calls[1].0, "flatpak");
        assert_eq!(r.calls[1].1, vec!["run", "md.obsidian.Obsidian"]);
    }

    #[test]
    fn execute_open_steam_spawns_steam() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::OpenApp(App::Steam));
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "steam");
    }

    #[test]
    fn steam_fallbacks_to_flatpak_when_direct_spawn_fails() {
        let mut r = FakeRunner {
            fail_steam: true,
            ..Default::default()
        };
        let keep = execute_with(&mut r, Command::OpenApp(App::Steam));
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 2);
        assert_eq!(r.calls[0].0, "steam");
        assert_eq!(r.calls[1].0, "flatpak");
        assert_eq!(r.calls[1].1, vec!["run", "com.valvesoftware.Steam"]);
    }

    #[test]
    fn execute_audio_pause_calls_playerctl() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::AudioPause);
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "playerctl");
        assert_eq!(r.calls[0].1, vec!["play-pause"]);
    }

    #[test]
    fn execute_audio_next_calls_playerctl() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::AudioNext);
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "playerctl");
        assert_eq!(r.calls[0].1, vec!["next"]);
    }

    #[test]
    fn execute_audio_previous_calls_playerctl() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::AudioPrevious);
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls[1].0, "playerctl");
        assert_eq!(r.calls[1].1, vec!["previous"]);
    }

    #[test]
    fn execute_audio_max_calls_wpctl() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::VolumeMax);
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "wpctl");
        assert_eq!(
            r.calls[0].1,
            vec!["set-volume", "@DEFAULT_AUDIO_SINK@", "100%"]
        );
    }

    #[test]
    fn execute_brightness_max_calls_brightnessctl() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::BrightnessMax);
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "brightnessctl");
        assert_eq!(r.calls[0].1, vec!["set", "100%"]);
    }

    #[test]
    fn execute_brightness_min_calls_brightnessctl() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::BrightnessMin);
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "brightnessctl");
        assert_eq!(r.calls[0].1, vec!["set", "5%"]);
    }

    #[test]
    fn execute_brightness_up_calls_brightnessctl() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::BrightnessUp);
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "brightnessctl");
        assert_eq!(r.calls[0].1, vec!["set", "10%+"]);
    }

    #[test]
    fn execute_brightness_down_calls_brightnessctl() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::BrightnessDown);
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "brightnessctl");
        assert_eq!(r.calls[0].1, vec!["set", "10%-"]);
    }

    #[test]
    fn execute_poweroff_calls_shutdown() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::Poweroff);
        assert_eq!(keep, CommandResult::Quit);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "poweroff");
    }

    #[test]
    fn execute_reboot_calls_reboot() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::Reboot);
        assert_eq!(keep, CommandResult::Quit);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "reboot");
    }

    #[test]
    fn execute_sleep_calls_systemctl() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::Sleep);
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "systemctl");
        assert_eq!(r.calls[0].1, vec!["suspend"]);
    }

    #[test]
    fn system_toggle_tests() {
        for toggle in SystemToggles::_iter() {
            let mut r = FakeRunner::default();
            match toggle {
                SystemToggles::Wifi => {
                    for _ in 0..=10 {
                        let mut r = FakeRunner::default();
                        let keep = execute_with(&mut r, Command::SystemToggle(SystemToggles::Wifi));

                        assert_eq!(keep, CommandResult::Running);

                        assert_eq!(r.calls.len(), 2);
                        assert_eq!(r.calls[0].0, "nmcli");
                        assert_eq!(r.calls[0].1, vec!["-t", "-f", "wifi", "radio"]);
                        assert_eq!(r.calls[1].0, "nmcli");
                        assert_eq!(r.calls[1].1[0..=1], vec!["radio", "wifi"]);
                    }
                }
                SystemToggles::Bluetooth => {
                    let keep =
                        execute_with(&mut r, Command::SystemToggle(SystemToggles::Bluetooth));

                    assert_eq!(keep, CommandResult::Running);

                    assert_eq!(r.calls.len(), 2);
                    assert_eq!(r.calls[0].0, "bluetooth");
                    assert_eq!(r.calls[1].0, "bluetoothctl");
                    assert_eq!(r.calls[1].1[0], "power");
                }
                SystemToggles::Volume => {
                    let keep = execute_with(&mut r, Command::SystemToggle(SystemToggles::Volume));

                    assert_eq!(keep, CommandResult::Running);

                    assert_eq!(r.calls.len(), 1);
                    assert_eq!(r.calls[0].0, "wpctl");
                    assert_eq!(
                        r.calls[0].1,
                        vec!["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]
                    );
                }
                SystemToggles::DoNotDisturb => {
                    let keep =
                        execute_with(&mut r, Command::SystemToggle(SystemToggles::DoNotDisturb));

                    assert_eq!(keep, CommandResult::Running);

                    assert_eq!(r.calls.len(), 1);
                    assert_eq!(r.calls[0].0, "qdbus6");
                    assert_eq!(
                        r.calls[0].1,
                        vec![
                            "org.kde.kglobalaccel",
                            "/component/plasmashell",
                            "org.kde.kglobalaccel.Component.invokeShortcut",
                            "\"toggle do not disturb\""
                        ]
                    );
                }
                SystemToggles::NightLight => {
                    let mut r = FakeRunner::default();
                    r.exec_output_values
                        .insert("xsct".to_string(), "6500".to_string());
                    let keep =
                        execute_with(&mut r, Command::SystemToggle(SystemToggles::NightLight));

                    assert_eq!(keep, CommandResult::Running);
                    assert_eq!(r.calls.len(), 2);

                    assert_eq!(r.calls[0].0, "xsct");
                    assert_eq!(r.calls[0].1, Vec::<String>::new());

                    assert_eq!(r.calls[1].0, "xsct");
                    assert_eq!(r.calls[1].1, vec!["4500"]);

                    let mut r = FakeRunner::default();
                    r.exec_output_values
                        .insert("xsct".to_string(), "4500".to_string());
                    let keep =
                        execute_with(&mut r, Command::SystemToggle(SystemToggles::NightLight));

                    assert_eq!(keep, CommandResult::Running);
                    assert_eq!(r.calls.len(), 2);
                    assert_eq!(r.calls[0].0, "xsct");
                    assert_eq!(r.calls[0].1, Vec::<String>::new());
                    assert_eq!(r.calls[1].0, "xsct");
                    assert_eq!(r.calls[1].1, vec!["6500"]);
                }
            }
        }
    }

    #[test]
    fn execute_open_terminal_spawns_kitty() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::OpenApp(App::Terminal));
        assert_eq!(keep, CommandResult::Running);
        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "kitty");
    }

    #[test]
    fn execute_screenshot_spawns_spectacle() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::Screenshot);
        assert_eq!(keep, CommandResult::Running);
        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "spectacle");
    }

    #[test]
    fn execute_volume_down_calls_wpctl() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::VolumeDown);
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "wpctl");
        assert_eq!(
            r.calls[0].1,
            vec!["set-volume", "@DEFAULT_AUDIO_SINK@", "5%-"]
        );
    }

    #[test]
    fn execute_end_conversation_returns_end_conversation() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::EndConversation);
        assert_eq!(keep, CommandResult::EndConversation);
        assert!(r.calls.is_empty());
    }

    #[test]
    fn execute_unknown_returns_running() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::Unknown("test".to_string()));
        assert_eq!(keep, CommandResult::Running);
        assert!(r.calls.is_empty());
    }

    #[test]
    fn execute_audio_previous_calls_playerctl_twice() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::AudioPrevious);
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 2);
        assert_eq!(r.calls[0].0, "playerctl");
        assert_eq!(r.calls[0].1, vec!["previous"]);
        assert_eq!(r.calls[1].0, "playerctl");
        assert_eq!(r.calls[1].1, vec!["previous"]);
    }

    #[test]
    fn execute_find_in_internet_opens_browser() {
        let mut r = FakeRunner::default();
        let prompt = "rust programming".to_string();
        let result = execute_with(&mut r, Command::FindInInternet(prompt.clone()));
        assert_eq!(result, CommandResult::Running);
        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "xdg-open");
        assert!(r.calls[0].1[0].contains("https://www.google.com/search?q="));
        assert!(r.calls[0].1[0].contains("rust"));
        assert!(r.calls[0].1[0].contains("programming"));
    }

    #[test]
    fn execute_switch_workspace_switches_workspace() {
        let mut r = FakeRunner {
            enviroment: String::from("Hyprland"),
            ..Default::default()
        };
        let keep = execute_with(&mut r, Command::SwitchWorkspace(7));
        assert_eq!(keep, CommandResult::Running);
        assert_eq!(r.calls.len(), 2);
        assert_eq!(
            r.calls[1].0,
            "~/.config/hypr/hyprland/scripts/workspace_action.sh"
        );
        assert_eq!(r.calls[1].1, vec!["workspace", "7"]);
    }

    #[test]
    fn random_tests_for_switch_workspace() {
        for i in 0..100 {
            let enviroment = match random::<bool>() {
                true => String::from("Hyprland"),
                false => String::from("KDE"),
            };
            let mut r = FakeRunner {
                enviroment,
                ..Default::default()
            };
            let keep = execute_with(&mut r, Command::SwitchWorkspace(7));
            assert_eq!(keep, CommandResult::Running);
            assert_eq!(r.calls.len(), 2);

            println!(
                "{i}\n{}\n{}\n{:?}",
                r.enviroment, r.calls[1].0, r.calls[1].1
            );

            if r.enviroment.as_str() == "Hyprland" {
                assert_eq!(
                    r.calls[1].0,
                    "~/.config/hypr/hyprland/scripts/workspace_action.sh"
                );
                assert_eq!(r.calls[1].1, vec!["workspace", "7"]);
            } else {
                assert_eq!(r.calls[1].0, "qdbus6");
                assert_eq!(
                    r.calls[1].1,
                    vec!["org.kde.KWin", "/KWin", "setCurrentDesktop", "7"]
                );
            }
        }
    }
}
