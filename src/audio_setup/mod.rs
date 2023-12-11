use audio_visualizer::dynamic::live_input::{setup_audio_input_loop, AudioDevAndCfg};
use cpal::{Device, Stream};
use ringbuffer::AllocRingBuffer;
use std::env;
use std::sync::{Arc, Mutex};
use cpal::traits::{HostTrait, DeviceTrait};

pub fn select_input_dev() -> Device {
    let mut devs = list_input_devs();
    assert!(!devs.is_empty(), "no input devices found!");
    // If there is only one input device, use it
    if devs.len() == 1 {
        return devs.remove(0).1;
    }

    // Display available input devices
    println!("Available input devices:");
    for (index, (name, _)) in devs.iter().enumerate() {
        println!("  [{}] {}", index, name);
    }

    // Grab device index from environment variable
    let index = env::var("DEVICE_INDEX")
        .expect("Please set the DEVICE_INDEX environment variable")
        .parse::<usize>()
        .expect("Invalid DEVICE_INDEX environment variable");
    devs.remove(index).1
}

pub fn setup_input_config() -> AudioDevAndCfg {
    let in_dev = select_input_dev();
    let input_dev_and_cfg = AudioDevAndCfg::new(Some(in_dev), None);
    input_dev_and_cfg
}

pub fn setup_live_input(
    latest_audio_data: Arc<Mutex<AllocRingBuffer<f32>>>,
    input_dev_and_cfg: AudioDevAndCfg,
) -> Stream {
    let stream = setup_audio_input_loop(latest_audio_data.clone(), input_dev_and_cfg);
    stream
}

pub fn list_input_devs() -> Vec<(String, cpal::Device)> {
    let host = cpal::default_host();
    type DeviceName = String;
    let mut devs: Vec<(DeviceName, Device)> = host
        .input_devices()
        .unwrap()
        .map(|dev| {
            (
                dev.name().unwrap_or_else(|_| String::from("<unknown>")),
                dev,
            )
        })
        .collect();
    devs.sort_by(|(n1, _), (n2, _)| n1.cmp(n2));
    devs
}
