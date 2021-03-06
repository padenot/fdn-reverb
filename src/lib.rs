pub mod allpass;
pub mod biquad;
pub mod delay_line;
pub mod filter;
pub mod softclip;
pub mod onepolelowpass;
pub mod utils;

use crate::allpass::Allpass;
use crate::delay_line::DelayLine;
use crate::filter::Filter;
use crate::onepolelowpass::OnePoleLowPass;
use crate::softclip::Softclip;
use crate::utils::{coprime_with_progression, hadamard, matrix_vector_multiply};
use crate::utils::clamp;

pub struct FDNReverb {
    drywet: f32,
    pre_delay: DelayLine,
    // four all pass
    all_passes: [Allpass; 4],
    // four delay lines
    delays: [DelayLine; 4],
    feedback: [f32; 4],
    feedback_matrix: [f32; 16],
    feedback_amount: f32,
    softclip: Softclip,
    lowpasses: [OnePoleLowPass; 4],
    sample_rate: f32,
    size: f32,
    progression: f32,
    width: f32
}

impl FDNReverb {
    pub fn new(sample_rate: f32) -> FDNReverb {
        let feedback = [0.; 4];
        let size = 300.;
        let delay_time = (size * sample_rate / 1000.) as u64;
        let allpass_time = (size * sample_rate / 1000.) as u64;
        let progression = 1.16;
        let delay_times = coprime_with_progression(delay_time as u64, progression, 4);
        let allpass_times = coprime_with_progression(allpass_time, progression, 4);

        println!("{:?}", delay_times.iter().map(|t| *t as f32 / sample_rate * 1000.).collect::<Vec::<f32>>());
        println!("{:?}", allpass_times.iter().map(|t| *t as f32 / sample_rate * 1000.).collect::<Vec::<f32>>());

        let mut pre_delay = DelayLine::new((150. * sample_rate / 1000.) as usize);
        pre_delay.set_duration(0);

        let all_passes = [
            Allpass::new(allpass_times[0] as f32 / sample_rate, 0.6, sample_rate),
            Allpass::new(allpass_times[1] as f32 / sample_rate, 0.6, sample_rate),
            Allpass::new(allpass_times[2] as f32 / sample_rate, 0.6, sample_rate),
            Allpass::new(allpass_times[3] as f32 / sample_rate, 0.6, sample_rate),
        ];
        let mut delays = [
            DelayLine::new(sample_rate as usize),
            DelayLine::new(sample_rate as usize),
            DelayLine::new(sample_rate as usize),
            DelayLine::new(sample_rate as usize),
        ];
        for (d, t) in delays.iter_mut().zip(delay_times) {
            d.set_duration(t as usize);
        }
        let mut feedback_matrix = [0.0; 16];
        feedback_matrix.copy_from_slice(&hadamard(4).unwrap());
        let halfsqrt2 = 1.; //(2.0 as f32).sqrt() / 2.;
        feedback_matrix.iter_mut().for_each(|c| *c *= halfsqrt2);

        // let mut feedback_matrix = [
        //     0., 1.,  1., 0.,
        //    -1., 0.,  0.,-1.,
        //     1., 0.,  0., -1.,
        //     0., 1., -1., 0.
        // ];
        // let lowpasses = [
        //     Filter::lowpass(2500., 0.5f32.sqrt(), sample_rate),
        //     Filter::lowpass(2500., 0.5f32.sqrt(), sample_rate),
        //     Filter::lowpass(2500., 0.5f32.sqrt(), sample_rate),
        //     Filter::lowpass(2500., 0.5f32.sqrt(), sample_rate),
        // ];
        let lowpasses = [
            OnePoleLowPass::new(2500., sample_rate),
            OnePoleLowPass::new(2500., sample_rate),
            OnePoleLowPass::new(2500., sample_rate),
            OnePoleLowPass::new(2500., sample_rate)
        ];

        return FDNReverb {
            pre_delay,
            drywet: 0.3,
            all_passes,
            delays,
            feedback_matrix,
            feedback,
            softclip: Softclip::new(1.25),
            lowpasses,
            feedback_amount: 0.8,
            sample_rate,
            size,
            progression,
            width: 1.0
        };
    }
    // [0, 1000]
    pub fn set_size(&mut self, size: f32) {
        println!("room size {}", size);
        self.size = size;
        // size in meter
        let s = if size < 1. { 1. } else { size };
        let duration_to_wall_s = s / 330.;
        let duration_to_wall_frames = (duration_to_wall_s / 5. * self.sample_rate) as u64;
        let duration_to_wall_frames_2 = (duration_to_wall_s / 35. * self.sample_rate) as u64;
        let progression = coprime_with_progression(duration_to_wall_frames, self.progression, 4);
        let progression_2 = coprime_with_progression(duration_to_wall_frames_2, self.progression, 4);
        println!("delays {:?}", progression.iter().map(|t| *t as f32 / self.sample_rate * 1000.).collect::<Vec::<f32>>());
        println!("allpasses {:?}", progression_2.iter().map(|t| *t as f32 / self.sample_rate * 1000.).collect::<Vec::<f32>>());
        // all passes are kept below 30ms
        for (ap, v) in self.all_passes.iter_mut().zip(progression_2.iter()) {
            ap.set_delay((*v) as f32);
        }
        for (d, v) in self.delays.iter_mut().zip(progression.iter()) {
            d.set_duration((*v) as usize);
        }
    }

    // [0, 1.25]
    pub fn set_pre_delay(&mut self, pre_delay: f32) {
        let pre_delay_frames = (pre_delay * self.sample_rate / 1000.) as usize;
        println!("pre-delay: {}", pre_delay);
        self.pre_delay.set_duration(pre_delay_frames);
    }
    // [0, 1.25]
    pub fn set_decay(&mut self, decay: f32) {
        println!("feedback: {}", decay);
        self.feedback_amount = decay;
        for a in self.all_passes.iter_mut() {
            a.set_gain(clamp(decay, 0.0, 0.6));
        }
    }
    // [0, 20000]
    pub fn set_absorbtion(&mut self, abs: f32) {
        for f in self.lowpasses.iter_mut() {
            f.set_frequency(abs);
        }
    }

    // [1, 2]
    pub fn set_hardness(&mut self, hardness: f32) {
        self.softclip.set_hardness(hardness);
    }

    pub fn set_progression(&mut self, progression: f32) {
        self.progression = progression;
        self.set_size(self.size);
    }

    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }

    pub fn set_drywet(&mut self, drywet: f32) {
        println!("drywet: {}", drywet);
        self.drywet = drywet;
    }
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let mut idx = 0;
        for ii in 0..input.len() {
            let mut a = [0.; 4];
            let mut b = [0.; 4];
            let mut predelayed = 0.0;

            self.pre_delay.process(input[ii], &mut predelayed);

            for i in 0..4 {
                self.lowpasses[i].process(predelayed + self.feedback[i], &mut a[i]);
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

            a = matrix_vector_multiply(&b, &self.feedback_matrix.into());

            for i in 0..4 {
                self.feedback[i] = a[i] * self.feedback_amount;
            }

            let mut l = input[ii] * (1.0 - self.drywet) + self.drywet * (self.feedback[0] + self.feedback[2]);
            let mut r = input[ii] * (1.0 - self.drywet) + self.drywet * (self.feedback[1] + self.feedback[3]);

            let mid = (l + r) / 2.;
            let mut side = (l - r) / 2.;

            side *= self.width;

            l = mid + side;
            r = mid - side;

            output[idx] = l;
            output[idx + 1] = r;
            idx += 2;
        }
    }
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn tail_size(&self) -> isize {
        // arbitrary
        (self.sample_rate * self.feedback_amount * 10.) as isize
    }
}

impl Default for FDNReverb {
    fn default() -> Self {
        FDNReverb::new(44100.)
    }
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn it_works() {}
}
