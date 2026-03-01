use crate::commands::CommandResult;

use super::{App, Command};

pub trait Runner {
    fn spawn(&mut self, program: &str, args: &[&str]) -> bool;
}

pub struct SystemRunner;

impl Runner for SystemRunner {
    fn spawn(&mut self, program: &str, args: &[&str]) -> bool {
        std::process::Command::new(program)
            .args(args)
            .spawn()
            .is_ok()
    }
}

pub fn execute_with<R: Runner>(runner: &mut R, cmd: Command) -> CommandResult {
    match cmd {
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
            find_in_internet(&prompt);
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
        Command::VolumeMute => {
            set_volume_mute(runner);
            CommandResult::Running
        }
        Command::VolumeMax => {
            set_volume(runner, "100%");
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
        Command::Quit => CommandResult::Quit,
        Command::Unknown(_text) => CommandResult::Running,
    }
}

fn open_app<R: Runner>(runner: &mut R, app: App) {
    match app {
        App::Firefox => {
            runner.spawn("firefox", &[]);
        }
        App::Terminal => {
            runner.spawn("ghostty", &[]);
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

fn poweroff<R: Runner>(runner: &mut R) {
    runner.spawn("poweroff", &[]);
}

fn reboot<R: Runner>(runner: &mut R) {
    runner.spawn("reboot", &[]);
}

fn sleep<R: Runner>(runner: &mut R) {
    runner.spawn("systemctl", &["suspend"]);
}

fn find_in_internet(prompt: &String) {
    let _ = open::that(format!("https://www.google.com/search?q={}", prompt));
}

fn screenshot<R: Runner>(runner: &mut R) {
    runner.spawn("spectacle", &[]);
}

fn set_volume<R: Runner>(runner: &mut R, delta: &str) {
    runner.spawn("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", delta]);
}

fn set_volume_mute<R: Runner>(runner: &mut R) {
    runner.spawn("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]);
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
    use super::*;
    use crate::commands::{App, Command};

    #[derive(Default)]
    struct FakeRunner {
        calls: Vec<(String, Vec<String>)>,
        fail_obsidian: bool,
        fail_telegram: bool,
        fail_telegram_desktop: bool,
        fail_steam: bool,
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
    fn execute_audio_mute_calls_wpctl() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::VolumeMute);
        assert_eq!(keep, CommandResult::Running);

        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "wpctl");
        assert_eq!(
            r.calls[0].1,
            vec!["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]
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
}
