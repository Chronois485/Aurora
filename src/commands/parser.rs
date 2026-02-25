use super::{App, Command};
use crate::normalizer::text::normalize;

pub fn parse_command(raw: &str) -> Command {
    let t = normalize(raw);

    if matches!(
        t.as_str(),
        "вихід" | "вимкнись" | "заверши роботу" | "стоп" | "stop" | "exit" | "quit"
    ) {
        return Command::Quit;
    }

    if has_any(
        &t,
        &[
            "досить",
            "все",
            "закінчимо",
            "that's all",
            "that's it",
            "nevermind",
            "bye",
        ],
    ) {
        return Command::EndConversation;
    }

    if has_any(&t, &["знайди ", "пошук ", "шукай ", "find ", "search "]) {
        for prefix in ["знайди ", "пошук ", "шукай ", "find ", "search "].iter() {
            if t.starts_with(prefix) {
                let query = t.trim_start_matches(prefix);
                return Command::FindInInternet(query.to_string());
            }
        }
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

    if has_any(
        &t,
        &[
            "постав на паузу",
            "пауза",
            "віднови",
            "зніми з паузи",
            "play",
            "pause",
        ],
    ) {
        return Command::AudioPause;
    }

    if has_any(&t, &["наступний", "наступна", "наступне", "next"]) {
        return Command::AudioNext;
    } else if has_any(
        &t,
        &[
            "минула",
            "минулий",
            "минуле",
            "минулі",
            "минуло",
            "previous",
        ],
    ) {
        return Command::AudioPrevious;
    }

    Command::Unknown(t)
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
    fn parse_audio_pause() {
        let cmd = parse_command("постав на паузу");
        assert!(matches!(cmd, Command::AudioPause));
    }

    #[test]
    fn parse_audio_next() {
        let cmd = parse_command("наступна пісня");
        assert!(matches!(cmd, Command::AudioNext));
    }

    #[test]
    fn parse_audio_previous() {
        let cmd = parse_command("минула пісня");
        assert!(matches!(cmd, Command::AudioPrevious));
    }

    #[test]
    fn unknown_keeps_text() {
        let cmd = parse_command("зроби мені чай");
        match cmd {
            Command::Unknown(t) => assert!(t.contains("зроби")),
            _ => panic!("expected Unknown"),
        }
    }

    #[test]
    fn parse_find_in_internet() {
        let cmd = parse_command("знайди парабола");
        if let Command::FindInInternet(query) = cmd {
            assert_eq!(query, "парабола");
        } else {
            panic!("expected FindInInternet");
        }
    }
}
