use fdn_reverb::delay_line::DelayLine;
use fdn_reverb::filter::Filter;
use fdn_reverb::utils::*;
use fdn_reverb::FDNReverb;
use std::fs::read_dir;

const BLOCK_SIZE: usize = 32;

fn main() {
    let reverb = FDNReverb::new(44100.);
    let start_in_frames = (10. / 1000. * 44100.) as u64;
    println!("start at 44.1klHz in frames is {}", start_in_frames);

    let v = coprime_with_progression(start_in_frames, 1.18, 16);

    println!("ms\tframes\tHz");
    for i in v.iter() {
        print!("{0:.2}\t", (*i as f32) * 1000. / 44100.);
        print!("{}\t", *i);
        print!("{0:.2}\n", 1. / ((*i as f32) / 44100.));
    }
    //let paths = read_dir("samples").unwrap();

    //let mut samples: Vec<Sample> = Vec::new();

    //for path in paths {
    //    let path = path.unwrap();
    //    samples.push(Sample::new(&path));
    //}

    //let s = &samples[0];
    //let mut d = DelayLine::new(44100);
    //d.set_duration(128);
    //let mut b = Filter::lowpass(1000., 10., 44100.);
    //let mut output_pcm = Vec::<i16>::with_capacity(s.frames());
    //output_pcm.resize(s.frames(), 0);

    //let mut i: usize = 0;
    //let mut output = Vec::<f32>::with_capacity(BLOCK_SIZE);
    //output.resize(BLOCK_SIZE, 0.);
    //let mut output2 = Vec::<f32>::with_capacity(BLOCK_SIZE);
    //output.resize(BLOCK_SIZE, 0.);
    //let mut j = 0;

    //loop {
    //    let input = s.slice(i, BLOCK_SIZE);
    //    i += input.len();
    //    if input.len() == 0 {
    //        break;
    //    }
    //    output.resize(input.len(), 0.);
    //    output2.resize(input.len(), 0.);
    //    b.process(&input, &mut output);
    //    d.process(&output, &mut output2);
    //    for i in output2.iter() {
    //        // clip and convert to 16bits
    //        let clipped;
    //        if *i > 0.9 {
    //            clipped = 0.9;
    //        } else if *i < -0.9 {
    //            clipped = -0.9;
    //        } else {
    //            clipped = *i;
    //        }
    //        let sample: i16 = (clipped * (2 << 14) as f32) as i16;
    //        output_pcm[j] = sample;
    //        j += 1;
    //    }
    //}
    //dump_wav("out.wav", &output_pcm, s.channels(), s.rate()).unwrap();
}
