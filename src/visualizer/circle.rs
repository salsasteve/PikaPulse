use std::f32::consts::PI;
use nannou::Draw;
use std::cell::Ref;
use nannou::prelude::Vec2;
use nannou::math::map_range;
use nannou::prelude::Rect;


pub fn sun(
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
    let spectrum_divider: usize = 8;
    let bins_used = num_bins / spectrum_divider;
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