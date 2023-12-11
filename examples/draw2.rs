use dotenv::dotenv;
use nannou::prelude::*;
use nannou::Draw;
use pika_pulse::recorder::Recorder;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use spectrum_analyzer::{
    samples_fft_to_spectrum, scaling::divide_by_N, windows::hann_window, FrequencyLimit,
    FrequencySpectrum, FrequencyValue,
};
use std::cell::Ref;
use std::cell::RefCell;
use std::cmp::max;
use std::sync::{Arc, Mutex};

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
    let mut recorder = Recorder::new();

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

    rainbow_circle3(&draw, &spectrum_data, 0.1, win, 3.0);

    draw.to_frame(app, &frame).unwrap();
}

/// Selects an audio input device based on user input.
// fn select_input_dev() -> cpal::Device {
//     let mut devs = list_input_devs();
//     assert!(!devs.is_empty(), "no input devices found!");
//     if devs.len() == 1 {
//         return devs.remove(0).1;
//     }

//     println!("Select an input device:");
//     devs.iter().enumerate().for_each(|(i, (name, dev))| {
//         println!("  [{}] {} {:?}", i, name, dev.default_input_config().unwrap());
//     });

//     let mut input = String::new();
//     stdin().lock().read_line(&mut input).unwrap();
//     let index = input[0..1].parse::<usize>().unwrap();
//     devs.remove(index).1
// }

/// Initializes a ring buffer for audio data storage.
fn init_ringbuffer(sampling_rate: usize) -> Arc<Mutex<AllocRingBuffer<f32>>> {
    let mut buf = AllocRingBuffer::new((5 * sampling_rate).next_power_of_two());
    buf.fill(0.0);
    Arc::new(Mutex::new(buf))
}

/// Processes audio data to generate a frequency spectrum.
fn to_spectrum(
    audio: &[f32],
    sampling_rate: f32,
    visualize_spectrum: &RefCell<Vec<(f64, f64)>>,
) -> Vec<(f64, f64)> {
    let relevant_samples = select_recent_samples(audio, 2048);

    let hann_window = hann_window(&relevant_samples);
    let latest_spectrum = perform_fft(&hann_window, sampling_rate);

    update_visualization(latest_spectrum, visualize_spectrum)
}

fn select_recent_samples(audio: &[f32], sample_count: usize) -> Vec<f32> {
    audio
        .iter()
        .skip(audio.len() - sample_count)
        .cloned()
        .collect()
}

fn perform_fft(samples: &[f32], sampling_rate: f32) -> FrequencySpectrum {
    samples_fft_to_spectrum(
        samples,
        sampling_rate as u32,
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

fn draw_visualization(
    draw: &Draw,
    spectrum: &Ref<Vec<(f64, f64)>>,
    amplitude: f32,
    radius: f32,
    number_of_points: usize,
) {
    // Clear the background
    draw.background().color(WHITE);

    // Set the center of the circle
    let center = vec2(0.0, 0.0);

    // Ensure we don't exceed the length of the spectrum data
    let spectrum_len = spectrum.len();
    let points_to_draw = number_of_points.min(spectrum_len / 2);

    // Iterate over the number of points to create the circle
    for i in 0..points_to_draw {
        // Assuming the second element of the tuple is the magnitude
        let (_, magnitude) = spectrum[i * 2]; // Adjust index based on your spectrum data
        let size = magnitude.powi(2) as f32; // Squaring the value for greater dynamic range

        // Calculate the angle for each point
        let angle = 2.0 * PI * i as f32 / number_of_points as f32;
        let (sin_angle, cos_angle) = angle.sin_cos();
        let inner_point = center + vec2(sin_angle, cos_angle) * radius;

        // Calculate the modifier based on the spectrum and amplitude
        let modifier = (1.0 + size / 2.0) * (1.0 + amplitude / 10.0);

        // Calculate the outer point of the line
        let outer_point = inner_point * modifier;

        // Set the stroke weight and color based on amplitude and position
        draw.line()
            .start(inner_point)
            .end(outer_point)
            .weight((amplitude + 1.0) * 10.0)
            .hsv(i as f32 / points_to_draw as f32, 1.0, 1.0); // Rainbow color
    }
}

fn rainbow_circle(
    draw: &Draw,
    spectrum: &Ref<Vec<(f64, f64)>>,
    amplitude: f32,
    radius: f32,
    win: Rect,
) {
    let num_bins = spectrum.len();
    let half_num_bins = num_bins / 2; // Use half the spectrum for a full circle
    const TWO_PI: f32 = 2.0 * PI;

    // Iterate over the spectrum data to create the circular visualization
    for (index, &(_, y)) in spectrum.iter().enumerate().take(half_num_bins) {
        // Map the index to an angle around the circle
        let angle = map_range(index, 0, half_num_bins, 0.0, TWO_PI);
        let (sin_angle, cos_angle) = angle.sin_cos();
        let inner_point = Vec2::new(sin_angle, cos_angle) * radius;

        // Scale the magnitude (y-value)
        let scaled_magnitude = map_range(y as f32, 0.0, 1.0, 0.0, win.h() / 2.0); // Assuming y in 0..1

        // Apply amplitude to scaling
        let modifier = (1.0 + scaled_magnitude / 2.0) * (1.0 + amplitude / 10.0);

        // Calculate the outer point of the line
        let outer_point = inner_point * modifier;

        // Set the stroke weight and color based on amplitude and position
        let hue = map_range(index, 0, half_num_bins, 0.0, 1.0);
        draw.line()
            .start(inner_point)
            .end(outer_point)
            .weight((amplitude + 1.0) * 10.0)
            .hsv(hue, 1.0, 1.0); // Rainbow color
    }
}

fn rainbow_circle2(draw: &Draw, spectrum: &Ref<Vec<(f64, f64)>>, amplitude: f32, win: Rect) {
    const TWO_PI: f32 = 2.0 * PI;
    let num_bins = spectrum.len();
    let half_num_bins = num_bins / 2; // Use half the spectrum for a full circle

    // Calculate the radius based on the smaller dimension of the window
    let radius = win.w().min(win.h()) * 0.1; // 40% of the smaller dimension

    // Iterate over the spectrum data to create the circular visualization
    for (index, &(_, y)) in spectrum.iter().enumerate().take(half_num_bins) {
        // Map the index to an angle around the circle
        let angle = map_range(index, 0, half_num_bins, 0.0, TWO_PI);
        let (sin_angle, cos_angle) = angle.sin_cos();
        let inner_point = Vec2::new(sin_angle, cos_angle) * radius;

        // Scale the magnitude (y-value)
        let scaled_magnitude = map_range(y as f32, 0.0, 1.0, 0.0, radius); // Scaled relative to radius

        // Apply amplitude to scaling
        let modifier = 1.0 + scaled_magnitude * (1.0 + amplitude);

        // Calculate the outer point of the line
        let outer_point =
            inner_point + Vec2::new(sin_angle, cos_angle) * scaled_magnitude * modifier;

        // Set the stroke weight and color based on amplitude and position
        let hue = map_range(index, 0, half_num_bins, 0.0, 1.0);
        draw.line()
            .start(inner_point + Vec2::new(win.w() / 2.0, win.h() / 2.0))
            .end(outer_point + Vec2::new(win.w() / 2.0, win.h() / 2.0))
            .weight((amplitude + 1.0) * 2.0)
            .hsv(hue, 1.0, 1.0); // Rainbow color
    }
}

fn rainbow_circle3(
    draw: &Draw,
    spectrum: &Ref<Vec<(f64, f64)>>,
    amplitude: f32,
    win: Rect,
    line_weight: f32,
) {
    const TWO_PI: f32 = 2.0 * PI;
    // number of bins to be used for the full circle
    // number of points to be used for the full circle
    let num_bins = spectrum.len();
    // number used to divide the spectrum
    let bin_divisor: usize = 8;
    let bins_used = num_bins / bin_divisor;
    // Calculate the radius based on the smaller dimension of the window
    let radius = win.w().min(win.h()) * 0.2; // 40% of the smaller dimension

    // Center of the window
    let center = Vec2::new(win.x(), win.y());

    // Iterate over the spectrum data to create the circular visualization
    for (index, &(_, y)) in spectrum.iter().enumerate().take(bins_used) {
        // Map the index to an angle around the circle
        let angle = map_range(index, 0, bins_used, 0.0, TWO_PI);
        let (sin_angle, cos_angle) = angle.sin_cos();
        let inner_point = center + Vec2::new(sin_angle, cos_angle) * radius;

        // Scale the magnitude (y-value)
        let scaled_magnitude = map_range(y as f32, 0.0, 1.0, 0.0, radius); // Scaled relative to radius

        // Calculate the outer point of the line
        let outer_point =
            inner_point + Vec2::new(sin_angle, cos_angle) * scaled_magnitude * amplitude;

        // Set the stroke weight and color based on amplitude and position
        let hue = map_range(index, 0, bins_used, 0.0, 1.0);
        draw.line()
            .start(inner_point)
            .end(outer_point)
            .weight(line_weight)
            .hsv(hue, 1.0, 1.0); // Rainbow color
    }
}
