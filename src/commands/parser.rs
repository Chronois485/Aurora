use super::{App, Command};

pub fn parse_command(raw: &str) -> Command {
    let t = normalize(raw);

    // вихід
    if matches!(t.as_str(), "вихід" | "вимкнись" | "заверши роботу" | "стоп") {
        return Command::Quit;
    }

    // гучність
    if has_any(&t, &["гучність", "звук", "громкість"]) {
        if has_any(&t, &["більше", "плюс", "вгору", "підніми", "додай"]) {
            return Command::VolumeUp;
        }
        if has_any(&t, &["менше", "мінус", "вниз", "зменш", "убав"]) {
            return Command::VolumeDown;
        }
    }

    // відкрий <...>
    if has_any(&t, &["відкрий", "запусти", "включи"]) {
        if has_any(&t, &["firefox", "файрфокс", "браузер", "ферфакс", "фаєр фокус", "фаєрфоксу"]) {
            return Command::OpenApp(App::Firefox);
        }
        if has_any(&t, &["термінал", "консоль", "командний рядок", "ghostty"]) {
            return Command::OpenApp(App::Terminal);
        }
        if has_any(&t, &["obsidian", "обсідіан", "нотатки"]) {
            return Command::OpenApp(App::Obsidian);
        }
    }

    Command::Unknown(t)
}

fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
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
        for phrase in ["вихід", "вимкнись", "заверши роботу", "стоп"] {
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