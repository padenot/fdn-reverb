use fdn_reverb::utils::dump_wav;
use fdn_reverb::biquad::Biquad;
use fdn_reverb::delay_line::DelayLine;
use std::fs::read_dir;
use fdn_reverb::Sample;

const BLOCK_SIZE: usize = 32;

fn main() {
    let paths = read_dir("samples").unwrap();

    let mut samples: Vec<Sample> = Vec::new();

    for path in paths {
        let path = path.unwrap();
        samples.push(Sample::new(&path));
    }

    let s = &samples[0];
    let mut d = DelayLine::new(44100);
    d.set_duration(128);
    let mut b = Biquad::low_pass(300., 2., 44100.);
    let mut output_pcm = Vec::<i16>::with_capacity(s.frames());
    output_pcm.resize(s.frames(), 0);

    let mut i: usize = 0;
    let mut output = Vec::<f32>::with_capacity(BLOCK_SIZE);
    output.resize(BLOCK_SIZE, 0.);
    let mut output2 = Vec::<f32>::with_capacity(BLOCK_SIZE);
    output.resize(BLOCK_SIZE, 0.);
    let mut j = 0;

    loop {
        let input = s.slice(i, BLOCK_SIZE);
        i += input.len();
        if input.len() == 0 {
            break;
        }
        output.resize(input.len(), 0.);
        output2.resize(input.len(), 0.);
        b.process(&input, &mut output);
        d.process(&output, &mut output2);
        for i in output2.iter() {
            // clip and convert to 16bits
            let clipped;
            if *i > 1.0 {
                clipped = 1.0;
            } else if *i < -1.0 {
                clipped = -1.0;
            } else {
                clipped = *i;
            }
            let sample: i16 = (clipped * (2 << 14) as f32) as i16;
            output_pcm[j] = sample;
            j += 1;
        }
    }
    dump_wav("out.wav", &output_pcm, s.channels(), s.rate()).unwrap();
}
