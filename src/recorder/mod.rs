use crate::audio_setup::{setup_input_config, setup_live_input};
use crate::utils::init_ringbuffer;
use cpal::traits::StreamTrait;
use cpal::Stream;
use ringbuffer::AllocRingBuffer;
use std::sync::{Arc, Mutex};

/// A struct that manages audio recording using CPAL.
///
/// The `Recorder` struct is responsible for handling audio recording, including
/// setting up the stream, storing the latest audio data, and controlling the recording process.
pub struct Recorder {
    stream: Stream,
    latest_audio_data: Arc<Mutex<AllocRingBuffer<f32>>>,
    sample_rate: f32,
}

impl Recorder {
    /// Constructs a new `Recorder` instance.
    ///
    /// Initializes the audio input configuration, sets up the live input stream, and
    /// prepares the buffer for storing the latest audio data.
    ///
    /// # Returns
    /// * `Recorder` - A new instance of `Recorder`.
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

    /// Starts the audio recording stream.
    ///
    /// Begins capturing audio data and storing it in the buffer.
    /// This method should be called when you want to start recording.
    pub fn start(&mut self) {
        self.stream.play().unwrap();
    }

    /// Pauses the audio recording stream.
    ///
    /// Stops capturing audio data without terminating the stream.
    /// This method can be used to temporarily halt recording.
    pub fn stop(&mut self) {
        self.stream.pause().unwrap();
    }

    /// Retrieves the sample rate of the audio stream.
    ///
    /// # Returns
    /// * `f32` - The sample rate at which the audio is being recorded.
    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Provides a clone of the Arc containing the latest audio data.
    ///
    /// This method can be used to access the audio data being captured by the recorder.
    ///
    /// # Returns
    /// * `Arc<Mutex<AllocRingBuffer<f32>>>` - A thread-safe reference to the buffer containing the latest audio data.
    pub fn get_latest_audio_data(&self) -> Arc<Mutex<AllocRingBuffer<f32>>> {
        self.latest_audio_data.clone()
    }
}