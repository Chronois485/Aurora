mod commands;

use commands::{executor, parser};

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use vosk::{DecodingState, Model, Recognizer};

fn main() -> Result<()> {
    let model = Model::new("models/small-uk-v3-normal").context("Vosk model not found")?;

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("No input device found")?;

    let supported = device.default_input_config()?;
    let config = supported.config();

    println!("Input device: {}", device.description()?);
    println!(
        "Format: channels={}, sample_rate={}, sample_format={:?}",
        config.channels,
        config.sample_rate,
        supported.sample_format()
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

    let sample_rate = config.sample_rate as f32;
    let mut rec = Recognizer::new(&model, sample_rate).context("Recognizer::new failed")?;

    let wake_word = "аврора";

    let command_window = Duration::from_secs(6);

    let mut armed = false;
    let mut armed_until = Instant::now();

    println!("Waiting for wake word...");

    loop {
        let chunk = rx.recv().context("Audio channel closed")?;

        let state = rec.accept_waveform(&chunk)?;

        if matches!(state, DecodingState::Finalized) {
            let res = rec.result();
            let text = match res {
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

            println!("final: {}", text);

            if !armed {
                if contains_wake(&text, wake_word) {
                    armed = true;
                    armed_until = Instant::now() + command_window;
                    println!("Wake word heard, say command...");
                    rec.reset();
                }
            } else {
                if Instant::now() <= armed_until {
                    println!("Command: {text}");
                    let cmd = parser::parse_command(text);
                    let keep_running = executor::execute(cmd);
                    if !keep_running {
                        return Ok(());
                    }
                } else {
                    println!("Timeout");
                }

                armed = false;
                rec.reset();
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
            // mono буфер
            let mut mono = Vec::with_capacity(data.len() / channels);

            for frame in data.chunks(channels) {
                // якщо стерео — усереднюємо L/R, якщо mono — беремо 0
                let sample = if channels >= 2 {
                    (frame[0] + frame[1]) * 0.5
                } else {
                    frame[0]
                };

                // f32 [-1.0..1.0] -> i16
                let s = sample.clamp(-1.0, 1.0);
                let i = (s * i16::MAX as f32) as i16;

                mono.push(i);
            }

            // відправляємо chunk у основний потік
            let _ = tx.send(mono);
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn contains_wake(text: &str, word: &str) -> bool {
    let t = text.to_lowercase();
    let w = word.to_lowercase();
    t.contains(&w)
}
