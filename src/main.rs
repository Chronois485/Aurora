use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use vosk::{Model, Recognizer};

fn main() {
    println!("Hello, world!");
}
