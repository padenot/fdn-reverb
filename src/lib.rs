pub mod biquad;
pub mod delay_line;
pub mod filter;
pub mod utils;

use crate::delay_line::DelayLine;
use crate::filter::Filter;
use crate::utils::{hadamard,coprime_with_progression, matrix_vector_multiply};

pub struct FDNReverb {
    // four all pass
    all_passes: [Filter; 4],
    // four delay lines
    delays: [DelayLine; 4],
    feedback: [f32; 4],
    feedback_matrix: [f32; 16],
}

impl FDNReverb {
    pub fn new(sample_rate: f32) -> FDNReverb {
        let feedback = [0.; 4];
        let ten_ms_in_frames = (40. * sample_rate / 1000.) as u64;
        let delay_times = coprime_with_progression(ten_ms_in_frames, 1.18, 4);
        let allpass_frequency: Vec::<f32> = delay_times.iter().map(|frames| 1. / (3. * (*frames as f32) / sample_rate)).collect();

        println!("delay_time: {:?}", delay_times);
        println!("allpass_frequency: {:?}", allpass_frequency);

        let mut all_passes = [
            Filter::allpass(1., 1., sample_rate),
            Filter::allpass(1., 1., sample_rate),
            Filter::allpass(1., 1., sample_rate),
            Filter::allpass(1., 1., sample_rate),
        ];
        for (d, f) in all_passes.iter_mut().zip(allpass_frequency) {
            d.set_frequency(f);
        }
        let mut delays = [
            DelayLine::new((sample_rate as usize) / 4),
            DelayLine::new((sample_rate as usize) / 4),
            DelayLine::new((sample_rate as usize) / 4),
            DelayLine::new((sample_rate as usize) / 4),
        ];
        for (d, t) in delays.iter_mut().zip(delay_times) {
            d.set_duration(t as usize);
        }
        let mut feedback_matrix = [0.0; 16];
        feedback_matrix.copy_from_slice(&hadamard(4).unwrap());
        return FDNReverb { all_passes, delays, feedback_matrix, feedback };
    }
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for (input, o) in input.iter().zip(output.iter_mut()) {
            let mut all_passed_samples = [0.; 4];
            let mut delayed_samples = [0.; 4];
            let mut diffused_samples : [f32; 4];
            for i in 0..4 {
                self.all_passes[i].process_single(*input + self.feedback[i], &mut all_passed_samples[i]);
                all_passed_samples[i] *= 0.6;
            }
            for i in 0..4 {
                self.delays[i].process_single(all_passed_samples[i], &mut delayed_samples[i]);
            }
            diffused_samples = matrix_vector_multiply(&delayed_samples, &self.feedback_matrix.into());
            for i in 0..4 {
                self.feedback[i] = diffused_samples[i] * 0.6;
            }
            *o = self.feedback.iter().sum::<f32>() / 4.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
