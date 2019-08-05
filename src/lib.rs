pub mod biquad;
pub mod utils;
pub mod delay_line; 
pub mod filter;

use audrey::*;
use log::*;
use std::fs::DirEntry;
use std::ops::Index;

pub struct Sample {
    name: String,
    channels: u32,
    rate: u32,
    data: Vec<f32>,
}

impl Sample {
    pub fn new(path: &DirEntry) -> Sample {
        info!("Loading {:?}...", path.path());
        let mut file = open(&path.path()).unwrap();
        let desc = file.description();
        let data: Vec<f32> = file.samples().map(Result::unwrap).collect::<Vec<_>>();
        let s = Sample {
            name: path.path().to_str().unwrap().to_string(),
            channels: desc.channel_count(),
            rate: desc.sample_rate(),
            data,
        };

        info!(
            "Loaded file: {} channels: {}, duration: {}, rate: {}",
            s.name(),
            s.channels(),
            s.duration(),
            s.rate()
        );

        return s;
    }
    pub fn channels(&self) -> u32 {
        self.channels
    }
    pub fn frames(&self) -> usize {
        self.data.len() / self.channels as usize
    }
    pub fn duration(&self) -> f32 {
        (self.data.len() as f32) / self.channels as f32 / self.rate as f32
    }
    pub fn rate(&self) -> u32 {
        self.rate
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn slice(&self, start: usize, size: usize) -> &[f32] {
        let mut real_size = size;
        if start + size >= self.data.len() {
            real_size = self.data.len() - start;
        }
        &self.data[start..start + real_size]
    }
}

impl Index<usize> for Sample {
    type Output = f32;

    fn index(&self, index: usize) -> &f32 {
        &self.data[index]
    }
}

// struct FDNReverb {
//     // four all pass
//     all_pass: [AllPass; 4],
//     // four delay lines
//     delay: [DelayLine; 4],
//     // four low pass
//     low_pass: [LowPass; 4]
// }
//
// impl FDNReverb {
//     fn new() -> FDNReverb {
//         let all_pass = {
//             AllPass::new(),
//             AllPass::new(),
//             AllPass::new(),
//             AllPass::new(),
//         }
//         let delay = {
//             DelayLine::new(),
//             DelayLine::new(),
//             DelayLine::new(),
//             DelayLine::new(),
//         }
//         let low_pass = {
//             LowPass::new(),
//             LowPass::new(),
//             LowPass::new(),
//             LowPass::new(),
//         }
//         return FDNReverb {
//             all_pass,delay,low_pass
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
