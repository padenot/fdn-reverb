pub mod biquad;
pub mod utils;
pub mod delay_line; 
pub mod filter;

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
