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

    if has_any(
        &t,
        &[
            "скріншот",
            "знімок екрана",
            "знімок екрану",
            "capture screen",
            "screenshot",
        ],
    ) {
        return Command::Screenshot;
    }

    if has_any(&t, &["знайди ", "пошук ", "шукай ", "find ", "search "]) {
        for prefix in ["знайди ", "пошук ", "шукай ", "find ", "search "].iter() {
            if t.starts_with(prefix) {
                let query = t.trim_start_matches(prefix);
                return Command::FindInInternet(query.to_string());
            }
        }
    }

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
        if has_any(&t, &["гучність", "звук", "громкість", "sound", "volume"]) {
            return Command::VolumeUp;
        }
        if has_any(&t, &["яркість", "яркість екрану", "brightness"]) {
            return Command::BrightnessUp;
        }
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
        if has_any(&t, &["гучність", "звук", "громкість", "sound", "volume"]) {
            return Command::VolumeDown;
        }
        if has_any(&t, &["яркість", "яркість екрану", "brightness"]) {
            return Command::BrightnessDown;
        }
    }

    if has_any(&t, &["minimum", "мінімум"]) {
        if has_any(&t, &["яркість", "яркість екрану", "brightness"]) {
            return Command::BrightnessMin;
        }
    }

    if has_any(&t, &["minimum", "мінімум", "mute", "заглуши"]) {
        if has_any(
            &t,
            &[
                "гучність",
                "звук",
                "громкість",
                "sound",
                "volume",
                "гучності",
            ],
        ) {
            return Command::VolumeMute;
        }
    }

    if has_any(&t, &["maximum", "максимум"]) {
        if has_any(&t, &["гучність", "звук", "громкість", "sound", "volume"]) {
            return Command::VolumeMax;
        }
        if has_any(&t, &["яркість", "яркість екрану", "brightness"]) {
            return Command::BrightnessMax;
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

    #[test]
    fn parse_find_in_internet_ua_variants() {
        for (input, expected) in [
            ("пошук рецепту борщу", "рецепту борщу"),
            ("шукай картинки котів", "картинки котів"),
            ("find rust programming", "rust programming"),
            ("search weather forecast", "weather forecast"),
        ] {
            if let Command::FindInInternet(query) = parse_command(input) {
                assert_eq!(query, expected, "failed for input: {input}");
            } else {
                panic!("expected FindInInternet for input: {input}");
            }
        }
    }

    #[test]
    fn parse_end_conversation() {
        for phrase in ["досить", "все", "закінчимо", "nevermind", "bye"] {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::EndConversation),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn parse_screenshot() {
        for phrase in [
            "скріншот",
            "знімок екрана",
            "знімок екрану",
            "capture screen",
            "screenshot",
        ] {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::Screenshot),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn parse_brightness_up() {
        for phrase in [
            "яркість більше",
            "яркість вгору",
            "brightness up",
            "підніми яркість",
        ] {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::BrightnessUp),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn parse_brightness_down() {
        for phrase in [
            "яркість менше",
            "яркість вниз",
            "brightness down",
            "зменш яркість",
        ] {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::BrightnessDown),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn parse_brightness_max() {
        for phrase in ["максимум яркість", "яркість максимум", "maximum brightness"]
        {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::BrightnessMax),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn parse_brightness_min() {
        for phrase in ["мінімум яркість", "яркість мінімум", "minimum brightness"]
        {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::BrightnessMin),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn parse_volume_mute() {
        for phrase in ["заглуши звук", "mute volume", "гучність на мінімум"]
        {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::VolumeMute),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn parse_volume_max() {
        for phrase in [
            "максимум гучність",
            "звук на максимум",
            "maximum volume",
            "гучність максимум",
        ] {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::VolumeMax),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn parse_open_obsidian() {
        for phrase in ["відкрий обсідіан", "запусти obsidian"] {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::OpenApp(App::Obsidian)),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn parse_quit_en_variants() {
        for phrase in ["stop", "exit", "quit"] {
            let cmd = parse_command(phrase);
            assert!(matches!(cmd, Command::Quit), "failed for phrase: {phrase}");
        }
    }

    #[test]
    fn edge_case_empty_input() {
        let cmd = parse_command("");
        assert!(matches!(cmd, Command::Unknown(_)));
    }

    #[test]
    fn edge_case_whitespace_only() {
        let cmd = parse_command("   ");
        assert!(matches!(cmd, Command::Unknown(_)));
    }

    #[test]
    fn edge_case_special_characters() {
        let cmd = parse_command("!!!@#$%^&*()");
        assert!(matches!(cmd, Command::Unknown(_)));
    }

    #[test]
    fn edge_case_mixed_case() {
        let cmd = parse_command("ВІДКРИЙ Firefox");
        assert!(matches!(cmd, Command::OpenApp(App::Firefox)));
    }

    #[test]
    fn edge_case_extra_whitespace() {
        let cmd = parse_command("   відкрий    термінал   ");
        assert!(matches!(cmd, Command::OpenApp(App::Terminal)));
    }

    #[test]
    fn volume_up_various_phrases() {
        for phrase in [
            "більше гучність",
            "плюс звук",
            "вгору гучність",
            "підніми звук",
            "вище звук",
            "up volume",
            "higher sound",
        ] {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::VolumeUp),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn volume_down_various_phrases() {
        for phrase in [
            "менше гучність",
            "мінус звук",
            "вниз гучність",
            "зменш звук",
            "убав гучність",
            "нижче звук",
            "down volume",
            "lower sound",
        ] {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::VolumeDown),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn audio_pause_variants() {
        for phrase in [
            "постав на паузу",
            "пауза",
            "віднови",
            "зніми з паузи",
            "play",
            "pause",
        ] {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::AudioPause),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn audio_next_variants() {
        for phrase in [
            "наступний трек",
            "наступна пісня",
            "наступне відео",
            "next track",
        ] {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::AudioNext),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn audio_previous_variants() {
        for phrase in [
            "минула пісня",
            "минулий трек",
            "минуле відео",
            "минулі треки",
            "previous track",
        ] {
            let cmd = parse_command(phrase);
            assert!(
                matches!(cmd, Command::AudioPrevious),
                "failed for phrase: {phrase}"
            );
        }
    }

    #[test]
    fn unknown_preserves_input() {
        let inputs = ["зроби мені чай", "яка погода", "розкажи жарт"];
        for input in inputs {
            if let Command::Unknown(text) = parse_command(input) {
                assert!(!text.is_empty(), "Unknown should be empty for: {input}");
            } else {
                panic!("expected Unknown for: {input}");
            }
        }
    }

    #[test]
    fn priority_quit_over_unknown() {
        let cmd = parse_command("вимкнись");
        assert!(matches!(cmd, Command::Quit));
    }

    #[test]
    fn priority_screenshot_over_unknown() {
        let cmd = parse_command("зроби скріншот будь ласка");
        assert!(matches!(cmd, Command::Screenshot));
    }
}
