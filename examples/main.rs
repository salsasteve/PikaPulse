use dotenv::dotenv;
use nannou;
use nannou::prelude::*;
use pika_pulse::recorder::Recorder;
use pika_pulse::visualizer::circle::sun;
use ringbuffer::RingBuffer;
use spectrum_analyzer::{
    samples_fft_to_spectrum, scaling::divide_by_N, FrequencyLimit,
    FrequencySpectrum, FrequencyValue,
};
use std::cell::RefCell;
use std::cmp::max;
use libm::cosf;

struct Model {
    _window: window::Id,
    recorder: Recorder,
    visualize_spectrum: RefCell<Vec<(f64, f64)>>,
}

fn main() {
    dotenv().ok();
    nannou::app(model).update(update).run();
}

/// Sets up the model for the Nannou application.
/// This includes creating a new window, selecting an audio input device,
/// configuring the audio stream, and initializing the data structures
/// for audio data and spectrum visualization.
fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build().unwrap();
    // let in_dev = select_input_dev();
    // let input_dev_and_cfg = AudioDevAndCfg::new(Some(in_dev), None);
    // let sample_rate = input_dev_and_cfg.cfg().sample_rate.0 as f32;
    // let latest_audio_data = init_ringbuffer(sample_rate as usize);
    let mut recorder = Recorder::new(None, None, None, None);

    let visualize_spectrum: RefCell<Vec<(f64, f64)>> = RefCell::new(vec![(0.0, 0.0); 1024]);

    // Setting up and playing the audio input stream.
    // let stream = setup_audio_input_loop(latest_audio_data.clone(), input_dev_and_cfg);
    // stream.play().unwrap();
    recorder.start();

    Model {
        _window,
        recorder,
        visualize_spectrum,
    }
}

/// Updates the model by processing the latest audio data and generating
/// the corresponding spectrum data for visualization.
fn update(_app: &App, model: &mut Model, _update: Update) {
    let latest_audio_data = model
        .recorder
        .get_latest_audio_data()
        .lock()
        .unwrap()
        .to_vec();
    let sample_rate = model.recorder.get_sample_rate();
    let spectrum_data = to_spectrum(&latest_audio_data, sample_rate, &model.visualize_spectrum);
    *model.visualize_spectrum.borrow_mut() = spectrum_data;
}

/// The view function for the Nannou application.
/// It draws the visual elements including the spectrum visualization.
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

    let num_bins = spectrum_data.len();

    // Update scaling logic
    let scaled_points = spectrum_data.iter().enumerate().map(|(index, &(_, y))| {
        // Map the index of each frequency bin to the x-axis (-512 to 512)
        let scaled_x = map_range(index, 0, num_bins - 1, win.left(), win.right());

        // Scale the y-axis magnitude. You need to adjust these values based on your data
        // Assuming 'y' is already scaled appropriately for your visualization
        let scaled_y = y as f32; // Assuming 'y' is within the range -384 to 384

        pt2(scaled_x, scaled_y)
    });

    // Draw the spectrum as a polyline
    draw.polyline().points(scaled_points).color(PALEGOLDENROD);

    sun(&draw, &spectrum_data, 0.1, win, 3.0);

    draw.to_frame(app, &frame).unwrap();
}

/// Processes audio data to generate a frequency spectrum.
fn to_spectrum(
    audio: &[i16],
    sampling_rate: u32,
    visualize_spectrum: &RefCell<Vec<(f64, f64)>>,
) -> Vec<(f64, f64)> {
    let relevant_samples = select_recent_samples(audio, 2048);

    let hann_window = hann_window(&relevant_samples);
    let latest_spectrum = perform_fft(&hann_window, sampling_rate);

    update_visualization(latest_spectrum, visualize_spectrum)
}

fn select_recent_samples(audio: &[i16], sample_count: usize) -> Vec<i16> {
    audio
        .iter()
        .skip(audio.len() - sample_count)
        .cloned()
        .collect()
}

fn perform_fft(samples: &[f32], sampling_rate: u32) -> FrequencySpectrum {
    samples_fft_to_spectrum(
        samples,
        sampling_rate,
        FrequencyLimit::All,
        Some(&divide_by_N),
    )
    .unwrap()
}

fn update_visualization(
    new_spectrum: FrequencySpectrum,
    visualize_spectrum: &RefCell<Vec<(f64, f64)>>,
) -> Vec<(f64, f64)> {
    // Descriptive constants
    const SMOOTHING_FACTOR: f64 = 0.84;
    const MAGNITUDE_SCALING_FACTOR: f32 = 5000.0;

    new_spectrum
        .data()
        .iter()
        .zip(visualize_spectrum.borrow_mut().iter_mut())
        .for_each(|((_, fr_val_new), (_, fr_val_old))| {
            // Apply a smoothing factor to the old magnitude value
            let old_val = *fr_val_old * SMOOTHING_FACTOR;

            // Scale the new magnitude value and compare it with the smoothed old value
            let scaled_new_val = *fr_val_new * MAGNITUDE_SCALING_FACTOR.into();
            let max_val = max(
                FrequencyValue::from(scaled_new_val),
                FrequencyValue::from(old_val as f32),
            );

            // Update the magnitude value in the visualization data
            *fr_val_old = max_val.val() as f64;
        });

    visualize_spectrum.borrow().clone()
}


pub fn hann_window(samples: &[i16]) -> Vec<f32> {
    let mut windowed_samples = Vec::with_capacity(samples.len());
    let samples_len_f32 = samples.len() as f32;
    for (i, sample) in samples.iter().enumerate() {
        let two_pi_i = 2.0 * PI * i as f32;
        let idontknowthename = cosf(two_pi_i / samples_len_f32);
        let multiplier = 0.5 * (1.0 - idontknowthename);
        windowed_samples.push(multiplier * i16_to_f32(*sample))
    }
    windowed_samples
}

pub fn i16_to_f32(sample: i16) -> f32 {
    sample as f32 / i16::MAX as f32 
}