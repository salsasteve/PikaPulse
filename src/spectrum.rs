use std::f32::consts::PI;
use libm::cosf;

const TWO_PI: f32 = PI * 2.0;

pub fn i16_to_f32(sample: i16) -> f32 {
    sample as f32 / i16::MAX as f32
}

pub fn apply_hann_window(samples: &[i16]) -> Option<Vec<f32>> {

    if samples.is_empty() {
        return None;
    }

    let windowed_samples: Vec<f32> = samples.iter()
        .enumerate()
        .map(|(i, sample)| hann(sample, samples.len() as f32, i as f32))
        .collect();

    Some(windowed_samples)
}

pub fn hann(sample: &i16, samples_len: f32, i: f32) -> f32 {
    let sample_f32 = i16_to_f32(*sample);
    let two_pi_i = TWO_PI * i;
    let cos_value = cosf(two_pi_i / samples_len);
    let multiplier = 0.5 * (1.0 - cos_value);
    multiplier * sample_f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    const EPSILON: f32 = 0.01;

    #[test]
    fn test_i16_to_f32() {
        assert_eq!(i16_to_f32(0), 0.0);
        assert_abs_diff_eq!(i16_to_f32(i16::MAX / 2), 0.5, epsilon = EPSILON);
        assert_abs_diff_eq!(i16_to_f32(i16::MIN), -1.0, epsilon = EPSILON);
    }

    #[test]
    fn test_hann() {
        let samples_len = 5.0;
        let samples = vec![0, i16::MAX, i16::MIN, i16::MAX / 2, i16::MIN / 2];
        let hann_output: Vec<f32> = samples.iter().enumerate().map(|(i, sample)| hann(sample, samples_len, i as f32)).collect();
        
        let mock_output = samples.iter().enumerate().map(|(i, sample)| {
            let sample_f32 = i16_to_f32(*sample);
            let two_pi_i = TWO_PI * i as f32;
            let cos_value = cosf(two_pi_i / samples_len);
            let multiplier = 0.5 * (1.0 - cos_value);
            multiplier * sample_f32
        }).collect::<Vec<f32>>();


        for (hann_val, mock_val) in hann_output.iter().zip(mock_output.iter()) {
            assert_abs_diff_eq!(hann_val, mock_val, epsilon = EPSILON);
        }

    }

    #[test]
    fn test_apply_hann_window() {
        let samples = vec![0, i16::MAX, i16::MIN, i16::MAX / 2, i16::MIN / 2];
        let output = apply_hann_window(&samples).unwrap();

        assert_eq!(output.len(), samples.len());

        let mock_output = samples.iter().enumerate().map(|(i, sample)| hann(sample, samples.len() as f32, i as f32)).collect::<Vec<f32>>(); 

        for (output_val, mock_val) in output.iter().zip(mock_output.iter()) {
            assert_abs_diff_eq!(output_val, mock_val, epsilon = EPSILON);
        }
    }

    #[test]
    fn test_apply_hann_window_empty() {
        let samples: Vec<i16> = vec![];
        let windowed_samples = apply_hann_window(&samples);
        assert!(windowed_samples.is_none());
    }

    #[test]
    fn test_apply_hann_window_single_sample() {
        let samples = vec![i16::MAX];
        let windowed_samples = apply_hann_window(&samples);
        assert_eq!(windowed_samples.len(), 1);
        assert!((windowed_samples[0] - 0.5).abs() < 0.01);
    }
}