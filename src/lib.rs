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
    feedback_amount: f32,
    softclip: Softclip,
    lowpasses: [Filter; 4],
    sample_rate: f32,
}

impl FDNReverb {
    pub fn new(sample_rate: f32) -> FDNReverb {
        let feedback = [0.; 4];
        let delay_time = (35. * sample_rate / 1000.) as u64;
        let allpass_time = (10. * sample_rate / 1000.) as u64;
        let delay_times = coprime_with_progression(delay_time as u64, 1.38, 4);
        let allpass_times = coprime_with_progression(allpass_time, 1.38, 4);

        println!("{:?}", delay_times);
        println!("{:?}", allpass_times);

        let all_passes = [
            Allpass::new(allpass_times[0] as f32 / sample_rate, 0.4, sample_rate),
            Allpass::new(allpass_times[1] as f32 / sample_rate, 0.4, sample_rate),
            Allpass::new(allpass_times[2] as f32 / sample_rate, 0.4, sample_rate),
            Allpass::new(allpass_times[3] as f32 / sample_rate, 0.4, sample_rate),
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
        let lowpasses = [
            Filter::lowpass(3000., 1.0, sample_rate),
            Filter::lowpass(3000., 1.0, sample_rate),
            Filter::lowpass(3000., 1.0, sample_rate),
            Filter::lowpass(3000., 1.0, sample_rate)
        ];
        feedback_matrix.iter_mut().for_each(|c| *c *= 0.701);

        return FDNReverb {
            all_passes,
            delays,
            feedback_matrix,
            feedback,
            softclip: Softclip::new(3.),
            lowpasses,
            feedback_amount: 0.5,
            sample_rate,
        };
    }
    // [0, 1000]
    pub fn set_size(&mut self, size: f32) {
        // size in meter
        let duration_to_wall_s = (size / 330.);
        let duration_to_wall_frames = (duration_to_wall_s * self.sample_rate) as u64;
        let progression = coprime_with_progression(duration_to_wall_frames, 1.18, 4);
        for (ap, v) in self.all_passes.iter_mut().zip(progression.iter()) {
            ap.set_delay(*v as f32 * self.sample_rate);
        }
        for (d, v) in self.delays.iter_mut().zip(progression.iter()) {
            d.set_duration(*v as usize);
        }
        println!("times: {:?}", progression.iter().map(|t| (*t as f32 / self.sample_rate)*1000.).collect::<Vec<f32>>());
    }
    // [0, 1.25]
    pub fn set_decay(&mut self, decay: f32) {
        println!("feedback: {}", decay);
        self.feedback_amount = decay;
    }
    // [0, 1]
    pub fn set_absorbtion(&mut self, abs: f32) {
        let c = 500. + abs * 20_000.;
        for f in self.lowpasses.iter_mut() {
            f.set_frequency(c);
        }
        println!("frequency: {}", c);
    }
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for (input, o) in input.iter().zip(output.iter_mut()) {
            let mut a = [0.; 4];
            let mut b = [0.; 4];
            for i in 0..4 {
                self.lowpasses[i].process((*input + self.feedback[i]), &mut a[i]);
            }
            for i in 0..4 {
                self.all_passes[i].process(a[i], &mut b[i]);
            }
            for i in 0..4 {
               self.delays[i].process(b[i], &mut a[i]);
            }
            for i in 0..4 {
                self.softclip.process(a[i], &mut b[i]);
            }

            b = matrix_vector_multiply(&a, &self.feedback_matrix.into());

            for i in 0..4 {
                self.feedback[i] = b[i] * self.feedback_amount;
            }

            *o = self.feedback.iter().sum::<f32>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
