use audio_visualizer::dynamic::live_input::setup_audio_input_loop;
use audio_visualizer::dynamic::live_input::{list_input_devs, AudioDevAndCfg};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::sync::{Arc, Mutex};
use std::io::{stdin, BufRead};
use cpal::traits::DeviceTrait;
use cpal::traits::StreamTrait;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, FrequencyValue};
use spectrum_analyzer::scaling::divide_by_N;
use std::cell::RefCell;
use std::cmp::max;

fn main() {
    let in_dev = select_input_dev();
    let input_dev_and_cfg = AudioDevAndCfg::new(Some(in_dev), None);
    let sample_rate = input_dev_and_cfg.cfg().sample_rate.0 as f32;
    let latest_audio_data = init_ringbuffer(sample_rate as usize);
    let stream = setup_audio_input_loop(latest_audio_data.clone(), input_dev_and_cfg);

    // This will be 1/44100 or 1/48000; the two most common sampling rates.
    let time_per_sample = 1.0 / sample_rate as f64;

    let visualize_spectrum: RefCell<Vec<(f64, f64)>> = RefCell::new(vec![(0.0, 0.0); 1024]);

    // Closure that captures `visualize_spectrum`.
    let to_spectrum_fn = move |audio: &[f32], sampling_rate: f32| {
        let skip_elements = audio.len() - 2048;
        // spectrum analysis only of the latest 46ms
        let relevant_samples = &audio[skip_elements..skip_elements + 2048];

        // do FFT
        let hann_window = hann_window(relevant_samples);
        let latest_spectrum = samples_fft_to_spectrum(
            &hann_window,
            sampling_rate as u32,
            FrequencyLimit::All,
            Some(&divide_by_N),
        )
        .unwrap();

        // now smoothen the spectrum; old values are decreased a bit and replaced,
        // if the new value is higher
        latest_spectrum
            .data()
            .iter()
            .zip(visualize_spectrum.borrow_mut().iter_mut())
            .for_each(|((fr_new, fr_val_new), (fr_old, fr_val_old))| {
                // actually only required in very first iteration
                *fr_old = fr_new.val() as f64;
                let old_val = *fr_val_old * 0.84;
                let max = max(
                    *fr_val_new * 5000.0_f32.into(),
                    FrequencyValue::from(old_val as f32),
                );
                *fr_val_old = max.val() as f64;
            });

        visualize_spectrum.borrow().clone()
    };

    stream.play().unwrap();
    // let latest_audio_data = latest_audio_data.clone().lock().unwrap().to_vec();
    // let data = to_spectrum_fn(&latest_audio_data, sample_rate);
    // println!("{:?}", data);
    // open_window_connect_audio(
    //     "Live Spectrum View",
    //     None,
    //     None,
    //     // 0.0..22050.0_f64.log(100.0),
    //     Some(0.0..8000.0),
    //     Some(0.0..500.0),
    //     "x-axis",
    //     "y-axis",
    //     AudioDevAndCfg::new(Some(in_dev), None),
    //     TransformFn::Complex(&to_spectrum_fn),
    // );

    // stream.pause().unwrap();

    
}

fn select_input_dev() -> cpal::Device {
    let mut devs = list_input_devs();
    assert!(!devs.is_empty(), "no input devices found!");
    if devs.len() == 1 {
        return devs.remove(0).1;
    }
    println!();
    devs.iter().enumerate().for_each(|(i, (name, dev))| {
        println!(
            "  [{}] {} {:?}",
            i,
            name,
            dev.default_input_config().unwrap()
        );
    });
    let mut input = String::new();
    stdin().lock().read_line(&mut input).unwrap();
    let index = input[0..1].parse::<usize>().unwrap();
    devs.remove(index).1
}

/// Inits a ringbuffer on the heap and fills it with zeroes.
fn init_ringbuffer(sampling_rate: usize) -> Arc<Mutex<AllocRingBuffer<f32>>> {
    // Must be a power (ringbuffer requirement).
    let mut buf = AllocRingBuffer::new((5 * sampling_rate).next_power_of_two());
    buf.fill(0.0);
    Arc::new(Mutex::new(buf))
}

