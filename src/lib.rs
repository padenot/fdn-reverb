pub mod biquad;
pub mod delay_line;
pub mod filter;
pub mod utils;
pub mod softclip; 
pub mod allpass;

use crate::delay_line::DelayLine;
use crate::filter::Filter;
use crate::utils::{coprime_with_progression, hadamard, matrix_vector_multiply};
use crate::softclip::Softclip;
use crate::allpass::Allpass;

pub struct FDNReverb {
    // four all pass
    all_passes: [Allpass; 4],
    // four delay lines
    delays: [DelayLine; 4],
    feedback: [f32; 4],
    feedback_matrix: [f32; 16],
    softclip: Softclip
}

impl FDNReverb {
    pub fn new(sample_rate: f32) -> FDNReverb {
        let feedback = [0.; 4];
        let ten_ms_in_frames = (18. * sample_rate / 1000.) as u64;
        let delay_times = coprime_with_progression(2 * ten_ms_in_frames, 1.7, 4);
        let allpass_times = coprime_with_progression(ten_ms_in_frames, 1.18, 4);
        let allpass_frequency: Vec<f32> = delay_times
            .iter()
            .map(|frames| 1. / (2. * (*frames as f32) / sample_rate))
            .collect();

        println!("delay_time: {:?}", delay_times.iter().map(|t| (*t as f32 / sample_rate)*1000.).collect::<Vec<f32>>());

        let mut all_passes = [
            Allpass::new(allpass_times[0] as f32 / sample_rate, 0.3, sample_rate),
            Allpass::new(allpass_times[1] as f32 / sample_rate, 0.3, sample_rate),
            Allpass::new(allpass_times[2] as f32 / sample_rate, 0.3, sample_rate),
            Allpass::new(allpass_times[3] as f32 / sample_rate, 0.3, sample_rate),
        ];
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
        //let mut feedback_matrix = [
        //    0., 1.,  1., 0.,
        //   -1., 0.,  0.,-1.,
        //    1., 0.,  0., -1.,
        //    0., 1., -1., 0.

        //];
        feedback_matrix.iter_mut().for_each(|c| *c *= 0.8);
        return FDNReverb {
            all_passes,
            delays,
            feedback_matrix,
            feedback,
            softclip: Softclip::new(0.4)
        };
    }
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for (input, o) in input.iter().zip(output.iter_mut()) {
            let mut all_passed_samples = [0.; 4];
            let mut delayed_samples = [0.; 4];
            let mut clipped = [0.; 4];
            let mut diffused_samples: [f32; 4];
            for i in 0..4 {
                self.all_passes[i]
                    .process((*input + self.feedback[i]), &mut all_passed_samples[i]);
            }
            for i in 0..4 {
               self.delays[i].process_single(all_passed_samples[i], &mut delayed_samples[i]);
            }

            for i in 0..4 {
                self.softclip.process(delayed_samples[i], &mut clipped[i]);
            }

            diffused_samples =
                matrix_vector_multiply(&clipped, &self.feedback_matrix.into());

            for i in 0..4 {
                self.feedback[i] = diffused_samples[i] * 0.5;
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
