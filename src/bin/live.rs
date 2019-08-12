use fdn_reverb::utils::*;
use fdn_reverb::FDNReverb;
use std::fs::read_dir;
use std::{thread, time};
use std::time::{Duration, Instant};

const BLOCK_SIZE: usize = 32;

struct LoopPlayer {
  sample: Sample,
  idx: usize
}

impl LoopPlayer {
    fn new(sample: Sample) -> LoopPlayer {
        LoopPlayer {
            sample,
            idx: 0
        }
    }
    fn extract(&mut self, frames: &mut [f32]) {
        for s in frames.iter_mut() {
            *s = self.sample.slice(self.idx, 1)[0];
            self.idx  = (self.idx + 1) % self.sample.frames();
        }
    }
}

fn main() {
    let paths = read_dir("samples").unwrap();

    let mut samples: Vec<Sample> = Vec::new();

    for path in paths {
        let path = path.unwrap();
        samples.push(Sample::new(&path));
    }

    let s = samples.pop().unwrap();
    let rate = s.rate();
    let mut loop_player = LoopPlayer::new(s);

    let mut reverb = FDNReverb::new(rate as f32);

    let ctx = cubeb::init("fdn-reverb").expect("Failed to create cubeb context");
    let params = cubeb::StreamParamsBuilder::new()
        .format(cubeb::SampleFormat::Float32NE)
        .rate(rate)
        .channels(1)
        .layout(cubeb::ChannelLayout::MONO)
        .take();

    let mut duration : u128 = 0;
    let mut callback_count = 0;
    let mut pcm = Vec::<f32>::with_capacity(1024);
    let mut wet = Vec::<f32>::with_capacity(1024);

    let mut builder = cubeb::StreamBuilder::new();
    builder
        .name("mlr-rs")
        .default_output(&params)
        .latency(256)
        .data_callback(move |_input: &[f32], mut output: &mut [f32]| {
            let start = Instant::now();
            pcm.resize(output.len(), 0.0);
            wet.resize(output.len(), 0.0);
            loop_player.extract(&mut pcm);
            reverb.process(&pcm, &mut wet);
            for i in 0..output.len() {
                output[i] = pcm[i] + wet[i] * 0.3;
            }
            let dd = start.elapsed();

            duration += dd.as_nanos();
            callback_count+=1;
            if callback_count % 100 == 0 {
                let avg_duration_us = (duration as f32 / callback_count as f32) / 1000.;
                let avg_duration_per_sample = (duration as f32 / callback_count as f32) / 1000. / 512.;
                let budget_us = 512. / 48000. * 1000. * 1000.;
                println!("{}us ({} per sample, dsp load: {})", avg_duration_us, avg_duration_per_sample, avg_duration_us/budget_us);
            }
            output.len() as isize
        })
        .state_callback(|state| {
            println!("stream {:?}", state);
        });

    let stream = builder.init(&ctx).expect("Failed to create cubeb stream");

    stream.start().unwrap();

    loop {
        let refresh = time::Duration::from_millis(10);
        thread::sleep(refresh);
    }
}
