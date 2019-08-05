pub mod biquad;
pub mod delay_line;
pub mod filter;
pub mod utils;

use crate::delay_line::DelayLine;
use crate::filter::Filter;

pub struct FDNReverb {
    // four all pass
    all_pass: [Filter; 4],
    // four delay lines
    delay: [DelayLine; 4],
}

impl FDNReverb {
    pub fn new(sample_rate: f32) -> FDNReverb {
        let all_pass = [
            Filter::allpass(1., 1., sample_rate),
            Filter::allpass(1., 1., sample_rate),
            Filter::allpass(1., 1., sample_rate),
            Filter::allpass(1., 1., sample_rate),
        ];
        let delay = [
            DelayLine::new(100),
            DelayLine::new(100),
            DelayLine::new(100),
            DelayLine::new(100),
        ];
        return FDNReverb { all_pass, delay };
    }
    fn process(&mut self, input: &[f32], output: &mut [f32]) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
