use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::WavReader;
use plotters::prelude::*;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

type StateHandle = Arc<Mutex<Option<(usize, Vec<f32>, Sender<()>)>>>;
fn main() {
    
}

fn write_output_data(output: &mut [f32], number_of_channels: usize, state: &StateHandle) {
    if let Ok(mut guard) = state.try_lock() {
        if let Some((i, samples, done)) = guard.as_mut() {
            for frame in output.chunks_mut(number_of_channels) {
                for (idx, sample) in frame.iter_mut().enumerate() {
                    *sample = *samples.get(*i+idx).unwrap_or(&0f32);
                }
                *i += number_of_channels;
            }
            if *i >= samples.len() {
                if let Err(_) = done.send(()) {
                    // Playback has already stopped. We'll be dead soon.
                }
            }
        }
    }
}

fn read_wav_file(file_path: &str) -> Vec<f32> {
    let mut reader = WavReader::open(file_path).expect("Failed to open WAV file");
    let samples = reader
        .samples::<f32>()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    samples
}

fn plot_samples(samples: &[f32], file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(file_name, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Waveform", ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..samples.len(), -1.0f32..1.0f32)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        samples.iter().enumerate().map(|(i, &v)| (i, v)),
        &RED,
    ))?;

    root.present()?;
    Ok(())
}


fn play_sample_cpal(){

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let output_conifg = device
        .default_output_config()
        .expect("Failed to get default configs");
    let number_of_channels = output_conifg.channels() as usize;
    println!("Default output config: {:?}", output_conifg);
    let wav_file_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "oli.wav");
    println!("{}", wav_file_path);

    let (done_tx, done_rx) = channel::<()>();
    let samples = read_wav_file(&wav_file_path);
    let state = (0 as usize, samples, done_tx);
    let state = Arc::new(Mutex::new(Some(state)));
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    // plot_samples(&samples, "output.png").expect("Failed to plot samples");

    let stream = device
        .build_output_stream(
            &output_conifg.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                write_output_data(data, number_of_channels, &state)
            },
            err_fn,
            None, // None=blocking, Some(Duration)=timeout
        )
        .expect("Error during build stream");

    stream.play().expect("Error during stream play");
    let _ = done_rx.recv();
}