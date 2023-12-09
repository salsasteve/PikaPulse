use cpal::Stream;
use std::sync::{Arc, Mutex};
use ringbuffer::AllocRingBuffer;
use crate::audio_setup::{setup_input_config, setup_live_input};
use crate::utils::init_ringbuffer;
use cpal::traits::StreamTrait;


pub struct Recorder {
    stream: Stream,
    latest_audio_data: Arc<Mutex<AllocRingBuffer<f32>>>,
    sample_rate: f32,
}

impl Recorder {
    pub fn new() -> Recorder {

        let dev_and_cfg = setup_input_config();
        let sample_rate = dev_and_cfg.cfg().sample_rate.0 as f32;
        let latest_audio_data = init_ringbuffer(sample_rate as usize);
        let stream = setup_live_input(latest_audio_data.clone(), dev_and_cfg);

        Recorder {
            stream,
            latest_audio_data,
            sample_rate,
        }
    } 
    pub fn start(&mut self) {
        self.stream.play().unwrap();
    }
    pub fn stop(&mut self) {
        self.stream.pause().unwrap();
    }
    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }
    pub fn get_latest_audio_data(&self) -> Arc<Mutex<AllocRingBuffer<f32>>> {
        self.latest_audio_data.clone()
    }
}


