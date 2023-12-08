use nannou::prelude::*;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use audio_visualizer::dynamic::live_input::setup_audio_input_loop;
use audio_visualizer::dynamic::live_input::{list_input_devs, AudioDevAndCfg};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::io::{stdin, BufRead};
use cpal::traits::DeviceTrait;
use cpal::traits::StreamTrait;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, FrequencyValue};
use spectrum_analyzer::scaling::divide_by_N;
use std::cmp::max;

struct Model {
    _window: window::Id,
    latest_audio_data: Arc<Mutex<AllocRingBuffer<f32>>>,
    visualize_spectrum: RefCell<Vec<(f64, f64)>>,
    sample_rate: f32,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {

    let _window = app.new_window().view(view).build().unwrap();
    let in_dev = select_input_dev();
    let input_dev_and_cfg = AudioDevAndCfg::new(Some(in_dev), None);
    let sample_rate = input_dev_and_cfg.cfg().sample_rate.0 as f32;
    let latest_audio_data = init_ringbuffer(sample_rate as usize);
    let stream = setup_audio_input_loop(latest_audio_data.clone(), input_dev_and_cfg);

    let visualize_spectrum: RefCell<Vec<(f64, f64)>> = RefCell::new(vec![(0.0, 0.0); 1024]);
    stream.play().unwrap();
    
    Model {
        _window,
        latest_audio_data,
        visualize_spectrum,
        sample_rate,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // Update the model's latest audio data with the latest audio data from the ringbuffer.
    let latest_audio_data = model.latest_audio_data.lock().unwrap().to_vec();
    let spectrum_data = to_spectrum(&latest_audio_data, model.sample_rate, &model.visualize_spectrum);
    *model.visualize_spectrum.borrow_mut() = spectrum_data;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(BLACK);
    // Draw a violet triangle
    let win = app.window_rect();
    draw.tri()
        .points(win.bottom_left(), win.top_left(), win.top_right())
        .color(VIOLET);

    // Draw the spectrum as a polyline
    let spectrum_data = model.visualize_spectrum.borrow();
    draw.polyline()
        .points(spectrum_data.iter().map(|&(x, y)| pt2(x as f32, y as f32)))
        .color(PALEGOLDENROD);

    draw.to_frame(app, &frame).unwrap();
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
// Assuming visualize_spectrum is a Vec<(f64, f64)>
fn to_spectrum(audio: &[f32], sampling_rate: f32, visualize_spectrum: &RefCell<Vec<(f64, f64)>>) -> Vec<(f64, f64)> {
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
            *fr_old = fr_new.val() as f64;
            let old_val = *fr_val_old * 0.84;
            let max = max(
                *fr_val_new * 5000.0_f32.into(),
                FrequencyValue::from(old_val as f32),
            );
            *fr_val_old = max.val() as f64;
        });

    visualize_spectrum.borrow().clone()
}

