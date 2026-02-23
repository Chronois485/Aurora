use super::{App, Command};

pub fn parse_command(raw: &str) -> Command {
    let t = normalize(raw);

    if matches!(
        t.as_str(),
        "вихід" | "вимкнись" | "заверши роботу" | "стоп" | "stop" | "exit" | "quit"
    ) {
        return Command::Quit;
    }

    if has_any(&t, &["гучність", "звук", "громкість", "sound", "volume"]) {
        if has_any(
            &t,
            &[
                "більше",
                "плюс",
                "вгору",
                "підніми",
                "додай",
                "вище",
                "up",
                "higher",
            ],
        ) {
            return Command::VolumeUp;
        }
        if has_any(
            &t,
            &[
                "менше",
                "мінус",
                "вниз",
                "зменш",
                "убав",
                "нижче",
                "down",
                "lower",
            ],
        ) {
            return Command::VolumeDown;
        }
    }

    if has_any(&t, &["відкрий", "запусти", "включи", "open", "launch"]) {
        if has_any(
            &t,
            &[
                "firefox",
                "файрфокс",
                "браузер",
                "ферфакс",
                "фаєр фокус",
                "фаєрфоксу",
                "browser",
                "internet browser",
            ],
        ) {
            return Command::OpenApp(App::Firefox);
        }
        if has_any(
            &t,
            &[
                "термінал",
                "консоль",
                "командний рядок",
                "ghostty",
                "terminal",
            ],
        ) {
            return Command::OpenApp(App::Terminal);
        }
        if has_any(&t, &["obsidian", "обсідіан", "нотатки", "notes"]) {
            return Command::OpenApp(App::Obsidian);
        }
        if has_any(
            &t,
            &[
                "ігри",
                "ігровий лаунчер",
                "steam",
                "стім",
                "games",
                "game launcher",
            ],
        ) {
            return Command::OpenApp(App::Steam);
        }
        if has_any(
            &t,
            &[
                "файли",
                "файловий менеджер",
                "дельфін",
                "провідник",
                "file explorer",
                "dolphin",
                "files",
                "explorer",
            ],
        ) {
            return Command::OpenApp(App::Dolphin);
        }
        if has_any(&t, &["telegram", "месенджер", "телеграм", "messenger"]) {
            return Command::OpenApp(App::Telegram);
        }
    }

    Command::Unknown(t)
}

pub fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn has_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| text.contains(n))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{App, Command};

    #[test]
    fn normalize_removes_punctuation_and_extra_spaces() {
        let s = "  Відкрий!!!   Firefox,  будь-ласка :)  ";
        assert_eq!(normalize(s), "відкрий firefox будь ласка");
    }

    #[test]
    fn parse_open_firefox_ua() {
        let cmd = parse_command("Відкрий файрфокс");
        assert!(matches!(cmd, Command::OpenApp(App::Firefox)));
    }

    #[test]
    fn parse_open_firefox_en() {
        let cmd = parse_command("запусти firefox");
        assert!(matches!(cmd, Command::OpenApp(App::Firefox)));
    }

    #[test]
    fn parse_open_terminal() {
        let cmd = parse_command("відкрий термінал");
        assert!(matches!(cmd, Command::OpenApp(App::Terminal)));
    }

    #[test]
    fn parse_open_telegram() {
        let cmd = parse_command("відкрий телеграм");
        assert!(matches!(cmd, Command::OpenApp(App::Telegram)));
    }

    #[test]
    fn parse_open_steam() {
        let cmd = parse_command("відкрий ігровий лаунчер");
        assert!(matches!(cmd, Command::OpenApp(App::Steam)));
    }

    #[test]
    fn parse_open_dolphin() {
        let cmd = parse_command("відкрий дельфін");
        assert!(matches!(cmd, Command::OpenApp(App::Dolphin)));
    }

    #[test]
    fn parse_volume_up() {
        let cmd = parse_command("гучність більше");
        assert!(matches!(cmd, Command::VolumeUp));
    }

    #[test]
    fn parse_volume_down() {
        let cmd = parse_command("зменш звук вниз");
        assert!(matches!(cmd, Command::VolumeDown));
    }

    #[test]
    fn parse_quit_variants() {
        for phrase in ["вихід", "вимкнись", "заверши роботу", "стоп"]
        {
            let cmd = parse_command(phrase);
            assert!(matches!(cmd, Command::Quit), "failed for phrase: {phrase}");
        }
    }

    #[test]
    fn unknown_keeps_text() {
        let cmd = parse_command("зроби мені чай");
        match cmd {
            Command::Unknown(t) => assert!(t.contains("зроби")),
            _ => panic!("expected Unknown"),
        }
    }
}
