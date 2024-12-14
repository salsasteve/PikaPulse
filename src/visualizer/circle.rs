use nannou::math::map_range;
use nannou::prelude::Rect;
use nannou::prelude::Vec2;
use nannou::Draw;
use std::cell::Ref;
use std::f32::consts::PI;

pub fn sun(
    draw: &Draw,
    spectrum: &Ref<Vec<(f64, f64)>>,
    amplitude: f32,
    win: Rect,
    line_weight: f32,
) {
    let bins_used = spectrum.len()/2;

    let radius = win.w().min(win.h()) * 0.2; // 40% of the smaller dimension

    // Center of the window
    let center = Vec2::new(win.x(), win.y());

    // Iterate over the spectrum data to create the circular visualization
    for (index, &(_, y)) in spectrum.iter().enumerate().take(bins_used) {
        // Get the angle in radians, sin and cos of the angle
        let (_angle, sin_angle, cos_angle) = calculate_index_to_angle(index, bins_used); 
        // println!("index: {}, angle: {}, sin: {}, cos: {}", index, _angle, sin_angle, cos_angle);  

        let inner_point = calculate_inner_point(center, radius, sin_angle, cos_angle);

        // println!("inner_point: {:?}", inner_point);

        // Scale the magnitude (y-value)

        let scaled_magnitude = scale_to_radius(y as f32, radius);

        // Calculate the outer point of the line
        let outer_point =
            inner_point + Vec2::new(sin_angle, cos_angle) * scaled_magnitude * amplitude;

        // Set the stroke weight and color based on amplitude and position
        let hue = calculate_hue(index, bins_used);
        draw.line()
            .start(inner_point)
            .end(outer_point)
            .weight(line_weight)
            .hsv(hue, 1.0, 1.0); // Rainbow color
    }
}

fn calculate_index_to_angle(index: usize, bins_used: usize) -> (f32, f32, f32) {
    const TWO_PI: f32 = 2.0 * PI;
    let mapped = map_range(index, 0, bins_used, 0.0, TWO_PI);
    let (sin_angle, cos_angle) = mapped.sin_cos();

    (mapped, sin_angle, cos_angle)
}

fn calculate_inner_point(center: Vec2, radius: f32, sin_angle: f32, cos_angle: f32) -> Vec2 {
    center + Vec2::new(sin_angle, cos_angle) * radius
}

fn calculate_hue(index: usize, bins_used: usize) -> f32 {
    map_range(index, 0, bins_used, 0.0, 1.0)
}

fn scale_to_radius(y: f32, radius: f32) -> f32 {
    map_range(y, 0.0, 1.0, 0.0, radius)
}

fn calculate_outer_point(center: Vec2, radius: f32, sin_angle: f32, cos_angle: f32, y: f32) -> Vec2 {
    let scaled_magnitude = scale_to_radius(y, radius);
    center + Vec2::new(sin_angle, cos_angle) * scaled_magnitude
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_range_start() {
        let index = 0;
        let bins_used = 1024;
        let (angle, sin_angle, cos_angle) = calculate_index_to_angle(index, bins_used);
        assert_eq!(angle, 0.0);
        assert!(sin_angle < 0.0001);
        assert!(cos_angle - 1.0 < 0.0001);
    }

    #[test]
    fn test_map_range_quarter() {
        let index = 256;
        let bins_used = 1024;
        let (angle, sin_angle, cos_angle) = calculate_index_to_angle(index, bins_used);
        assert_eq!(angle, PI / 2.0);
        assert!(sin_angle - 1.0 < 0.0001);
        assert!(cos_angle < 0.0001);
    }

    #[test]
    fn test_map_range_quarter2() {
        let index = 2;
        let bins_used = 8;
        let (angle, sin_angle, cos_angle) = calculate_index_to_angle(index, bins_used);
        assert_eq!(angle, PI / 2.0);
        assert!(sin_angle - 1.0 < 0.0001);
        assert!(cos_angle < 0.0001);
    }

    #[test]
    fn test_map_range_half() {
        let index = 512;
        let bins_used = 1024;
        let (angle, sin_angle, cos_angle) = calculate_index_to_angle(index, bins_used);
        assert_eq!(angle, PI);
        assert!(sin_angle < 0.0001);
        assert!(cos_angle - 1.0 < 0.0001);
    }

    #[test]
    fn test_map_range_half2() {
        let index = 4;
        let bins_used = 8;
        let (angle, sin_angle, cos_angle) = calculate_index_to_angle(index, bins_used);
        assert_eq!(angle, PI);
        assert!(sin_angle < 0.0001);
        assert!(cos_angle - 1.0 < 0.0001);
    }

    #[test]
    fn test_map_range_three_quarters() {
        let index = 768;
        let bins_used = 1024;
        let (angle, sin_angle, cos_angle) = calculate_index_to_angle(index, bins_used);
        assert_eq!(angle, 3.0 * PI / 2.0);
        assert!(sin_angle + 1.0 < 0.0001);
        assert!(cos_angle < 0.0001);
    }

    #[test]
    fn test_map_range_three_quarters2() {
        let index = 6;
        let bins_used = 8;
        let (angle, sin_angle, cos_angle) = calculate_index_to_angle(index, bins_used);
        assert_eq!(angle, 3.0 * PI / 2.0);
        assert!(sin_angle + 1.0 < 0.0001);
        assert!(cos_angle < 0.0001);
    }

    #[test]
    fn test_map_range_end() {
        let index = 1023;
        let bins_used = 1024;
        let (angle, sin_angle, cos_angle) = calculate_index_to_angle(index, bins_used);
        assert_eq!(angle, (2.0 * PI * 1023.0) / 1024.0);
        assert!(sin_angle < 0.0001);
        assert!(cos_angle - 1.0 < 0.0001);
    }

    #[test]
    fn test_map_range_end2() {
        let index = 7;
        let bins_used = 8;
        let (angle, sin_angle, cos_angle) = calculate_index_to_angle(index, bins_used);
        assert_eq!(angle, (2.0 * PI * 7.0) / 8.0);
        assert!(sin_angle < 0.0001);
        assert!(cos_angle - 1.0 < 0.0001);
    }

    #[test]
    fn test_calculate_inner_point() {
        let center = Vec2::new(0.0, 0.0);
        let radius = 1.0;
        let sin_angle = 0.0;
        let cos_angle = 1.0;
        let inner_point = calculate_inner_point(center, radius, sin_angle, cos_angle);
        assert_eq!(inner_point, Vec2::new(0.0, 1.0));
    }

    #[test]
    fn test_calculate_inner_point2() {
        let center = Vec2::new(1.0, 1.0);
        let radius = 1.0;
        let sin_angle = 0.0;
        let cos_angle = 1.0;
        let inner_point = calculate_inner_point(center, radius, sin_angle, cos_angle);
        assert_eq!(inner_point, Vec2::new(1.0, 2.0));
    }
}
