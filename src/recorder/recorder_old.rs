use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig, SupportedStreamConfig, default_host};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::io::BufWriter;
use std::thread::sleep;
use std::time::Duration;
use std::sync::mpsc::channel;

pub struct AudioClip {
    input_device: Device,
    output_device: Device,
    input_config: SupportedStreamConfig,
    output_config: SupportedStreamConfig,
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    samples: Arc<Mutex<Vec<f32>>>,
    is_recording: bool,
    is_playing: bool,
    current_position: usize,
    file_path: PathBuf,
    clip_length: Duration,
}

type WavWriterHandle = Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>;

impl AudioClip {
    pub fn new(clip_name: String, clip_length: u64) -> Result<Self, anyhow::Error> {
        let host = cpal::default_host();

        let file_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), clip_name);
        let input_device = host
            .default_input_device()
            .expect("No input device available");
        let output_device = host
            .default_output_device()
            .expect("No output device available");
        let input_config = input_device
            .default_input_config()
            .expect("Failed to get default input config");
        let output_config = output_device
            .default_output_config()
            .expect("Failed to get default input config");

        Ok(AudioClip {
            input_device,
            output_device,
            input_config,
            output_config,
            input_stream: None,
            output_stream: None,
            samples: Arc::new(Mutex::new(Vec::<f32>::new())),
            is_recording: false,
            is_playing: false,
            current_position: 0,
            file_path: file_path.into(),
            clip_length: Duration::from_secs(clip_length),
        })
    }

    pub fn record(&mut self) -> Result<(), anyhow::Error> {
        let writer = self.setup_writer_for_recording()?;

        self.setup_input_stream(&writer)?;

        println!("Recording Started");
        self.is_recording = true;
        self.start_stream1()?;

        sleep(self.clip_length);
        self.stop_stream1()?;

        self.finalize_writer_for_recording(writer)?;
        self.is_recording = false;
        println!("Recording Finished");

        let samples_len = {
            let samples_guard = self.samples.lock().unwrap();
            samples_guard.len()
        };

        println!("Recorded {} samples", samples_len);

        Ok(())
    }

    fn setup_writer_for_recording(&self) -> Result<WavWriterHandle, anyhow::Error> {
        let spec = AudioClip::wav_spec_from_config(&self.output_config);
        let writer = WavWriter::create(&self.file_path, spec)?;
        Ok(Arc::new(Mutex::new(Some(writer))))
    }

    fn read_wav_file(file_path: PathBuf) -> Result<Arc<Mutex<Vec<f32>>>, anyhow::Error> {
        let reader = WavReader::open(file_path)?;
        let samples = Arc::new(Mutex::new(reader
            .into_samples::<f32>()
            .filter_map(Result::ok)
            .collect()));
        Ok(samples)
    }

    fn setup_input_stream(&mut self, writer: &WavWriterHandle) -> Result<(), anyhow::Error> {
        let writer_clone = Arc::clone(&writer);
        let samples_clone = Arc::clone(&self.samples);
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        self.input_stream = Some(self.input_device.build_input_stream(
            &self.input_config.clone().into(),
            move |data: &[f32], _: &_| {
                AudioClip::write_input_data(data, &writer_clone, &samples_clone)
            },
            err_fn,
            None,
        )?);
        Ok(())
    }

    fn setup_output_stream(output_config: &StreamConfig, output_device: &Device, samples: Arc<Mutex<Vec<f32>>>) -> Result<Stream, anyhow::Error> {
        // let samples_clone = Arc::clone(&samples);
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        // let output_stream = output_device.build_output_stream(
        //     &output_config.clone().into(),
        //     move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        //         // Acquire a lock on the samples
        //         let samples_guard = &samples.lock().unwrap();

        //         // Iterate over the output buffer and the samples, cycling through the samples if needed
        //         for (output_sample, &wav_sample) in data.iter_mut().zip(samples_guard.iter().cycle()) {
        //             *output_sample = wav_sample;
        //         }
        //     },
        //     err_fn,
        //     None,
        // )?;
        let stream = output_device.build_output_stream(
            output_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let samples_guard = &samples.lock().unwrap();
                for (output_sample, &wav_sample) in data.iter_mut().zip(samples_guard.iter().cycle()) {
                    *output_sample = wav_sample;
                }
            },
            err_fn,
            None,
        )?;

        // let output_stream = output_device.build_output_stream(
        //     &output_config.clone().into(),
        //     move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        //         let mut ss = samples_clone.lock().unwrap();
        //         for s in data.iter_mut() {
        //             *s = ss.next().unwrap_or(0.0);
        //         }
        //     },
        //     err_fn,
        //     None,
        // )?;

        Ok(stream)
    }

    fn start_stream1(&self) -> Result<(), anyhow::Error> {
        if let Some(stream) = &self.input_stream {
            stream.play()?;
        }
        Ok(())
    }

    fn stop_stream1(&self) -> Result<(), anyhow::Error> {
        if let Some(stream) = &self.input_stream {
            stream.pause()?;
        }
        Ok(())
    }

    fn start_stream(stream: &Stream) -> Result<(), anyhow::Error> {
        stream.play()?;
        Ok(())
    }

    fn stop_stream(stream: &Stream) -> Result<(), anyhow::Error> {
        stream.pause()?;
        Ok(())
    }

    fn finalize_writer_for_recording(&self, writer: WavWriterHandle) -> Result<(), anyhow::Error> {
        writer.lock().unwrap().take().unwrap().finalize()?;
        Ok(())
    }

    fn write_input_data(input: &[f32], writer: &WavWriterHandle, samples: &Arc<Mutex<Vec<f32>>>) {
        if let Ok(mut guard) = writer.try_lock() {
            if let Some(writer) = guard.as_mut() {
                let mut samples_guard = samples.lock().unwrap();
                for &sample in input.iter() {
                    writer.write_sample(sample).ok();
                    samples_guard.push(sample);
                }
            }
        }
    }

    fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> WavSpec {
        WavSpec {
            channels: config.channels() as _,
            sample_rate: config.sample_rate().0 as _,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        }
    }
}
