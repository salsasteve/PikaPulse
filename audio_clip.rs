use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, Stream, SupportedStreamConfig, FromSample};
use hound::{WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Represents an audio clip that can be recorded from an input device.
pub struct AudioClip {
    device: Device,
    config: SupportedStreamConfig,
    writer: WavWriterHandle,
    stream: Option<Stream>,
    clip_name: String,
    clip_length: Duration,
    samples: Vec<f32>,
}

type WavWriterHandle = Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>;

impl AudioClip {
    /// Creates a new `AudioClip` instance for the specified device.
    ///
    /// # Arguments
    ///
    /// * `device_name` - The name of the audio device to use for recording.
    ///                   Pass "default" to use the system's default input device.
    ///
    /// # Returns
    ///
    /// A result containing the `AudioClip` instance or an error.
    pub fn new(device_name: &str, clip_name: String, clip_length: u32) -> Result<Self, anyhow::Error> {
        let host = cpal::default_host(); 
        let clip_name = clip_name.to_string();
        let device = if device_name == "default" {
            host.default_input_device()
        } else {
            host.input_devices()?
                .find(|x| x.name().map(|y| y == device_name).unwrap_or(false))
        }
        .expect("failed to find input device");

        let config = device
            .default_input_config()
            .expect("Failed to get default input config");
        
        let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), clip_name);
        let spec = wav_spec_from_config(&config);
        let writer = hound::WavWriter::create(&path, spec)?;
        let writer = Arc::new(Mutex::new(Some(writer)));
        let clip_length = Duration::from_secs(clip_length as u64);

        Ok(AudioClip {
            device,
            config,
            writer,
            stream: None,
            clip_name,
            clip_length,
            samples: Vec::new(),
        })
    }

    /// Starts recording audio from the input device.
    ///
    /// # Arguments
    ///
    /// * `duration` - Duration for how long the recording should last.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub fn record(&mut self) -> Result<(), anyhow::Error> {
        
        let writer_2 = self.writer.clone();

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        self.stream = Some(match self.config.sample_format() {
            cpal::SampleFormat::I8 => self.device.build_input_stream(
                &self.config.clone().into(),
                move |data, _: &_| write_input_data::<i8, i8>(data, &writer_2),
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => self.device.build_input_stream(
                &self.config.clone().into(),
                move |data, _: &_| write_input_data::<i16, i16>(data, &writer_2),
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I32 => self.device.build_input_stream(
                &self.config.clone().into(),
                move |data, _: &_| write_input_data::<i32, i32>(data, &writer_2),
                err_fn,
                None,
            )?,
            cpal::SampleFormat::F32 => self.device.build_input_stream(
                &self.config.clone().into(),
                move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
                err_fn,
                None,
            )?,
            _ => return Err(anyhow::Error::msg("Unsupported sample format")),
        });

        if let Some(stream) = &self.stream {
            stream.play()?;
        }

        std::thread::sleep(self.clip_length);

        Ok(())
    }

    /// Finalizes the recording and saves the audio clip to a file.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub fn finalize(&mut self) -> Result<(), anyhow::Error> {
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }

        self.writer.lock().unwrap().take().unwrap().finalize()?;
        Ok(())
    }
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> WavSpec {
    WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}