use audio_visualizer::dynamic::live_input::{setup_audio_input_loop, list_input_devs, AudioDevAndCfg};
use cpal::{Stream, Device};
use cpal::traits::DeviceTrait;
use std::io::{stdin,BufRead};
use std::sync::{Arc, Mutex};
use ringbuffer::AllocRingBuffer;


pub fn select_input_dev() -> Device {
    let mut devs = list_input_devs();
    assert!(!devs.is_empty(), "no input devices found!");
    if devs.len() == 1 {
        return devs.remove(0).1;
    }

    println!("Select an input device:");
    devs.iter().enumerate().for_each(|(i, (name, dev))| {
        println!("  [{}] {} {:?}", i, name, dev.default_input_config().unwrap());
    });

    let mut input = String::new();
    stdin().lock().read_line(&mut input).unwrap();
    let index = input[0..1].parse::<usize>().unwrap();
    devs.remove(index).1
}

pub fn setup_input_config() -> AudioDevAndCfg {
    let in_dev = select_input_dev();
    let input_dev_and_cfg = AudioDevAndCfg::new(Some(in_dev), None);
    input_dev_and_cfg
}

pub fn setup_live_input(latest_audio_data:Arc<Mutex<AllocRingBuffer<f32>>>, input_dev_and_cfg:AudioDevAndCfg) -> Stream {
    let stream = setup_audio_input_loop(latest_audio_data.clone(), input_dev_and_cfg);
    stream
}