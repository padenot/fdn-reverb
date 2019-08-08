use fdn_reverb::delay_line::DelayLine;
use fdn_reverb::filter::Filter;
use fdn_reverb::utils::*;
use fdn_reverb::FDNReverb;
use std::fs::read_dir;

const BLOCK_SIZE: usize = 32;

fn main() {
    let mut reverb = FDNReverb::new(44100.);

    let paths = read_dir("samples").unwrap();

    let mut samples: Vec<Sample> = Vec::new();

    for path in paths {
        let path = path.unwrap();
        samples.push(Sample::new(&path));
    }

    let s = &samples[0];
    let mut output_pcm = Vec::<i16>::with_capacity(s.frames());
    output_pcm.resize(s.frames(), 0);

    let mut i: usize = 0;
    let mut output = Vec::<f32>::with_capacity(BLOCK_SIZE);
    output.resize(BLOCK_SIZE, 0.);
    let mut j = 0;

    loop {
        let input = s.slice(i, BLOCK_SIZE);
        i += input.len();
        if input.len() == 0 {
            break;
        }
        output.resize(input.len(), 0.);
        reverb.process(&input, &mut output);
        for (i, o) in input.iter().zip(output.iter()) {
            // clip and convert to 16bits
            let clipped;
            if *o > 0.9 {
                clipped = 0.9;
            } else if *o < -0.9 {
                clipped = -0.9;
            } else {
                clipped = *o;
            }
            let sample: i16 = ((*i + 0.5 * clipped) * (2 << 14) as f32) as i16;
            output_pcm[j] = sample;
            j += 1;
        }
    }
    dump_wav("out.wav", &output_pcm, s.channels(), s.rate()).unwrap();
}
