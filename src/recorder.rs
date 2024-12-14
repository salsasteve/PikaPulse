use cpal::traits::StreamTrait;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Stream;
use cpal::{Device, StreamConfig};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::sync::{Arc, Mutex};

pub struct Recorder {
    host: cpal::Host,
    device: Device,
    config: StreamConfig,
    sample_bit_depth: u8,
    stream: Stream,
    latest_audio_data: Arc<Mutex<AllocRingBuffer<i16>>>,
    sample_rate: u32,
}

impl Recorder {
    pub fn new(
        preferred_dev: Option<cpal::Device>,
        preferred_cfg: Option<cpal::StreamConfig>,
        preferred_sample_rate: Option<u32>,
        preferred_bit_depth: Option<u8>,
    ) -> Recorder {
        let sample_rate: u32 = preferred_sample_rate.unwrap_or(48000);
        let sample_bit_depth: u8 = preferred_bit_depth.unwrap_or(16);

        let latest_audio_data = Recorder::init_ringbuffer(sample_rate as usize);

        let host = cpal::default_host();
        let device = preferred_dev.unwrap_or_else(|| {
            let devices: Vec<Device> = host.input_devices().unwrap().collect();
            if devices.is_empty() {
                panic!("No input devices found for host {}", host.id().name());
            }
            devices.into_iter().nth(0).unwrap()
        });

        let config = preferred_cfg.unwrap_or_else(|| {
            Recorder::find_supported_config(&device, sample_rate).unwrap_or_else(|| {
                panic!(
                    "No supported stream configuration found for device '{}'",
                    device.name().unwrap_or_else(|_| "<unknown>".to_string())
                )
            })
        });

        let stream = Recorder::setup_audio_input_loop(latest_audio_data.clone(), &device, &config);

        Recorder {
            host,
            device,
            config,
            sample_bit_depth,
            stream,
            latest_audio_data,
            sample_rate,
        }
    }

    fn find_supported_config(device: &Device, sample_rate: u32) -> Option<StreamConfig> {
        let configs = device.supported_input_configs().ok()?;
        for config in configs {
            if config.min_sample_rate().0 <= sample_rate
                && config.max_sample_rate().0 >= sample_rate
            {
                return Some(StreamConfig {
                    channels: config.channels(),
                    sample_rate: cpal::SampleRate(sample_rate),
                    buffer_size: cpal::BufferSize::Default,
                });
            }
        }
        None
    }

    pub fn init_ringbuffer(sampling_rate: usize) -> Arc<Mutex<AllocRingBuffer<i16>>> {
        let mut buf = AllocRingBuffer::new((5 * sampling_rate).next_power_of_two());
        buf.fill(0i16);
        Arc::new(Mutex::new(buf))
    }

    pub fn start(&mut self) {
        self.stream.play().unwrap();
    }

    pub fn stop(&mut self) {
        self.stream.pause().unwrap();
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }
    pub fn get_latest_audio_data(&self) -> Arc<Mutex<AllocRingBuffer<i16>>> {
        self.latest_audio_data.clone()
    }

    pub fn setup_audio_input_loop(
        latest_audio_data: Arc<Mutex<AllocRingBuffer<i16>>>,
        dev: &Device,
        cfg: &StreamConfig,
    ) -> cpal::Stream {
        eprintln!(
            "Using input device '{}' with config: {:?}",
            dev.name()
                .as_ref()
                .map(|x| x.as_str())
                .unwrap_or("<unknown>"),
            cfg
        );

        assert!(
            cfg.channels == 1 || cfg.channels == 2,
            "only supports Mono or Stereo channels!"
        );

        if cfg.sample_rate.0 != 44100 && cfg.sample_rate.0 != 48000 {
            eprintln!(
                "WARN: sampling rate is {}, but the crate was only tested with 44,1/48khz.",
                cfg.sample_rate.0
            );
        }

        let is_mono = cfg.channels == 1;

        let stream = dev
            .build_input_stream(
                // This is not as easy as it might look. Even if the supported configs show, that a
                // input device supports a given fixed buffer size, ALSA but also WASAPI tend to
                // fail with unclear error messages. I found out, that using the default option is the
                // only variant that is working on all platforms (Windows, Mac, Linux). The buffer
                // size tends to be not as small as it would be optimal (for super low latency)
                // but is still good enough (for example ~10ms on Windows) or ~6ms on ALSA (in my
                // tests).
                cfg,
                // this is pretty cool by "cpal"; we can use u16, i16 or f32 and
                // the type system does all the magic behind the scenes. f32 also works
                // on Windows (WASAPI), MacOS (coreaudio), and Linux (ALSA).
                // TODO: I found out that we probably can't rely on the fact, that every audio input device
                //  supports f32. I guess, I need to check this in the supported audio stream config too..
                move |data: &[i16], _info| {
                    let mut audio_buf = latest_audio_data.lock().unwrap();
                    // Audio buffer only contains Mono data
                    if is_mono {
                        audio_buf.extend(data.iter().copied());
                    } else {
                        // interleaving for stereo is LRLR (de-facto standard?)
                        audio_buf.extend(
                            data.chunks_exact(2)
                                .map(|vals| ((vals[0] as f32 + vals[1] as f32) / 2.0) as i16),
                        )
                    }
                },
                |err| {
                    eprintln!("got stream error: {:#?}", err);
                },
                None,
            )
            .unwrap();

        stream
    }
}
