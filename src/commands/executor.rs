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

pub fn execute_with<R: Runner>(runner: &mut R, cmd: Command) -> bool {
    match cmd {
        Command::OpenApp(app) => {
            open_app(runner, app);
            true
        }
        Command::VolumeUp => {
            set_volume(runner, "5%+");
            true
        }
        Command::VolumeDown => {
            set_volume(runner, "5%-");
            true
        }
        Command::Quit => false,
        Command::Unknown(_text) => true,
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
            if !runner.spawn("Telegram", &[]) {
                if !runner.spawn("telegram-desktop", &[]) {
                    runner.spawn("flatpak", &["run", "org.telegram.desktop"]);
                }
            }
        }
    }
}

fn set_volume<R: Runner>(runner: &mut R, delta: &str) {
    runner.spawn("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", delta]);
}

pub fn execute(cmd: Command) -> bool {
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
        assert!(keep);
        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "firefox");
    }

    #[test]
    fn execute_open_dolphin_spawns_dolphin() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::OpenApp(App::Dolphin));
        assert!(keep);
        assert_eq!(r.calls.len(), 1);
        assert_eq!(r.calls[0].0, "dolphin");
    }

    #[test]
    fn execute_open_telegram_spawns_telegram() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::OpenApp(App::Telegram));
        assert!(keep);
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
        assert!(keep);

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
        assert!(keep);

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
        assert!(keep);

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
        assert!(!keep);
        assert!(r.calls.is_empty());
    }

    #[test]
    fn execute_open_obsidian_spawns_obsidian() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::OpenApp(App::Obsidian));
        assert!(keep);
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
        assert!(keep);

        assert_eq!(r.calls.len(), 2);
        assert_eq!(r.calls[0].0, "obsidian");
        assert_eq!(r.calls[1].0, "flatpak");
        assert_eq!(r.calls[1].1, vec!["run", "md.obsidian.Obsidian"]);
    }

    #[test]
    fn execute_open_steam_spawns_steam() {
        let mut r = FakeRunner::default();
        let keep = execute_with(&mut r, Command::OpenApp(App::Steam));
        assert!(keep);

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
        assert!(keep);

        assert_eq!(r.calls.len(), 2);
        assert_eq!(r.calls[0].0, "steam");
        assert_eq!(r.calls[1].0, "flatpak");
        assert_eq!(r.calls[1].1, vec!["run", "com.valvesoftware.Steam"]);
    }
}
