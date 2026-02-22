use super::{App, Command};

pub trait Runner {
    fn spawn(&mut self, program: &str, args: &[&str]) -> bool;
}

pub struct SystemRunner;

impl Runner for SystemRunner {
    fn spawn(&mut self, program: &str, args: &[&str]) -> bool {
        std::process::Command::new(program).args(args).spawn().is_ok()
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
        Command::Unknown(_) => true,
    }
}

fn open_app<R: Runner>(runner: &mut R, app: App) {
    match app {
        App::Firefox => { runner.spawn("firefox", &[]); }
        App::Terminal => { runner.spawn("ghostty", &[]); }
        App::Obsidian => {
            if !runner.spawn("obsidian", &[]) {
                runner.spawn("flatpak", &["run", "md.obsidian.Obsidian"]);
            }
        }
    }
}

fn set_volume<R: Runner>(runner: &mut R, delta: &str) {
    runner.spawn("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", delta]);
}

// Зручний wrapper для продакшну:
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
    fn obsidian_fallbacks_to_flatpak_when_direct_spawn_fails() {
        let mut r = FakeRunner { fail_obsidian: true, ..Default::default() };
        let keep = execute_with(&mut r, Command::OpenApp(App::Obsidian));
        assert!(keep);

        assert_eq!(r.calls.len(), 2);
        assert_eq!(r.calls[0].0, "obsidian");
        assert_eq!(r.calls[1].0, "flatpak");
        assert_eq!(r.calls[1].1, vec!["run", "md.obsidian.Obsidian"]);
    }
}