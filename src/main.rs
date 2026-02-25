mod audio;
mod commands;
mod normalizer;
mod settings;

use anyhow::{Context, Result};
use audio::resample::LinearResampler;
use colored::Colorize;
use commands::{executor, parser::parse_command};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use normalizer::{audio::AudioNormalizer, text};
use settings::manager::{get_setting, print_settings};
use std::sync::mpsc;
use std::{
    io,
    time::{Duration, Instant},
};
use vosk::{DecodingState, LogLevel, Model, Recognizer, set_log_level};

const TARGET_SR: u32 = 16_000;
const SETTINGS_FILE_PATH: &str = "settings.json";

enum Languages {
    English,
    Ukrainian,
}

enum Models {
    Nano,
    Small,
    Normal,
}

fn main() -> Result<()> {
    let text_mode = match get_setting("text_mode", SETTINGS_FILE_PATH).as_str() {
        "true" => true,
        _ => false,
    };

    print_settings(SETTINGS_FILE_PATH);
    println!();

    if text_mode {
        loop {
            let mut cmd = String::new();
            println!("{}", "[*] Waiting for command...".cyan());
            io::stdin().read_line(&mut cmd)?;
            let cmd = parse_command(cmd.trim());
            println!(
                "{}",
                format!("[+] Recognized command {:?}", cmd).green().bold()
            );

            let keep_running = executor::execute(cmd);
            if !keep_running {
                return Ok(());
            }
        }
    } else {
        let model = match get_setting("model", SETTINGS_FILE_PATH).as_str() {
            "nano" => Models::Nano,
            "small" => Models::Small,
            "normal" => Models::Normal,
            model => {
                println!(
                    "{}\n{}\n{}",
                    format!("[!] Unknown model: {}\n", model).red(),
                    "[*] Available models: nano, small, normal".magenta(),
                    "[*] Using default model (Normal)".cyan()
                );
                Models::Normal
            }
        };

        let language = match get_setting("language", SETTINGS_FILE_PATH).as_str() {
            "uk" => Languages::Ukrainian,
            "en" => Languages::English,
            lang => {
                println!(
                    "{}\n{}\n{}",
                    format!("[!] Unknown language: {}\n", lang).red(),
                    "[*] Available languages: English, Ukrainian".magenta(),
                    "[*] Using default language (English)".cyan()
                );
                Languages::English
            }
        };

        let mut model_path = String::from("models/stt/");

        model_path.push_str(match language {
            Languages::English => "en",
            Languages::Ukrainian => "uk",
        });

        model_path.push('-');

        model_path.push_str(match model {
            Models::Nano => "nano",
            Models::Small => "small",
            Models::Normal => "normal",
        });

        set_log_level(LogLevel::Error);

        println!();
        println!("{}", "[*] Initializing...".magenta());

        let mut norm = AudioNormalizer::new();
        let model = Model::new(model_path).context("Vosk model not found")?;

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("No input device found")?;

        let supported = device.default_input_config()?;
        let config = supported.config();

        println!(
            "{}{}",
            "[*] Input device: ".magenta().bold(),
            format!("{}", device.description()?).magenta()
        );
        println!();
        println!(
            "{}\n{}\n{}\n{}",
            "[*] Format:".bold().magenta(),
            format!("    - channels={}", config.channels).magenta(),
            format!("    - sample_rate={}", config.sample_rate).magenta(),
            format!("    - sample_format={:?}", supported.sample_format()).magenta(),
        );

        let (tx, rx) = mpsc::channel::<Vec<i16>>();
        let channels = config.channels as usize;

        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => build_stream_f32(&device, &config, channels, tx)?,
            _ => anyhow::bail!(
                "Now supporting only f32. Your format: {:?}.",
                supported.sample_format()
            ),
        };

        stream.play()?;

        let mut rec =
            Recognizer::new(&model, TARGET_SR as f32).context("Recognizer::new failed")?;

        let input_sr = config.sample_rate;
        let mut rs = LinearResampler::new(input_sr, TARGET_SR);

        let wake_word = match language {
            Languages::English => "aurora",
            Languages::Ukrainian => "аврора",
        };
        let command_window = Duration::from_secs(6);

        let mut armed = false;
        let mut armed_until = Instant::now();

        println!("{}", "[+] Initialization complete!".green());

        println!();
        println!("{}", "[*] Waiting for wake word...".cyan().italic());

        loop {
            let mono_in = rx.recv().context("Audio channel closed")?;

            let mut chunk_16k = rs.process(&mono_in);

            if !norm.process(&mut chunk_16k) {
                continue;
            }

            let state = rec.accept_waveform(&chunk_16k)?;

            if matches!(state, DecodingState::Finalized) {
                let res = rec.result();
                let text: &str = match res {
                    vosk::CompleteResult::Single(single) => single.text,
                    vosk::CompleteResult::Multiple(multiple) => {
                        if let Some(first) = multiple.alternatives.first() {
                            first.text
                        } else {
                            ""
                        }
                    }
                };

                if text.is_empty() {
                    continue;
                }

                if !armed {
                    println!("{text}");
                    if contains_wake(text, wake_word) {
                        armed = true;
                        armed_until = Instant::now() + command_window;
                        println!("{}", "[+] Wake word heard, say command...".green().bold());
                        rec.reset();
                    }
                } else {
                    if Instant::now() <= armed_until {
                        println!("{}", format!("[*] Your command: {text}").cyan());
                        let cmd = parse_command(text);
                        println!(
                            "{}",
                            format!("[+] Recognized command: {:?}", cmd).green().bold()
                        );

                        let keep_running = executor::execute(cmd);
                        if !keep_running {
                            return Ok(());
                        }
                    } else {
                        println!("{}", "[!] Timeout".yellow());
                    }

                    armed = false;
                    rec.reset();
                }
            }
        }
    }
}

fn build_stream_f32(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    tx: mpsc::Sender<Vec<i16>>,
) -> Result<cpal::Stream> {
    let err_fn = |err| eprintln!("Stream error: {err}");

    let stream = device.build_input_stream(
        config,
        move |data: &[f32], _info| {
            let mut mono = Vec::with_capacity(data.len() / channels);

            for frame in data.chunks(channels) {
                let sample = if channels >= 2 {
                    (frame[0] + frame[1]) * 0.5
                } else {
                    frame[0]
                };

                let s = sample.clamp(-1.0, 1.0);
                let i = (s * i16::MAX as f32) as i16;
                mono.push(i);
            }

            let _ = tx.send(mono);
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn contains_wake(text: &str, word: &str) -> bool {
    let t = text::normalize(text);
    let w = text::normalize(word);
    t.contains(&w)
}
