use crossbeam::queue::ArrayQueue;
use cubeb::StereoFrame;
use fdn_reverb::utils::*;
use fdn_reverb::FDNReverb;
use monome::*;
use std::fs::read_dir;
use std::sync::Arc;
use std::time::Instant;
use std::{thread, time};
use std::io;

const PROFILE: bool = false;

struct LoopPlayer {
    sample: Sample,
    idx: usize,
}

impl LoopPlayer {
    fn new(sample: Sample) -> LoopPlayer {
        LoopPlayer { sample, idx: 0 }
    }
    fn extract(&mut self, frames: &mut [f32]) {
        for s in frames.iter_mut() {
            *s = self.sample.slice(self.idx, 1)[0];
            self.idx = (self.idx + 1) % self.sample.frames();
        }
    }
}

enum Parameter {
    Absorbtion(f32),
    Size(f32),
    Decay(f32),
    SaturationHardness(f32),
    Progression(f32),
    DryWet(f32)
}

fn main() {
    let q = Arc::new(ArrayQueue::<Parameter>::new(32));
    let q2 = q.clone();
    let paths = read_dir("samples").unwrap();

    let mut samples: Vec<Sample> = Vec::new();

    for path in paths {
        let path = path.unwrap();
        samples.push(Sample::new(&path));
    }

    let s = samples.pop().unwrap();
    let rate = 48000;
    let mut loop_player = LoopPlayer::new(s);

    let mut reverb = FDNReverb::new(rate as f32);

    let ctx = cubeb::init("fdn-reverb").expect("Failed to create cubeb context");
    let params = cubeb::StreamParamsBuilder::new()
        .format(cubeb::SampleFormat::Float32NE)
        .rate(rate)
        .channels(2)
        .layout(cubeb::ChannelLayout::STEREO)
        .take();

    let mut duration: u128 = 0;
    let mut callback_count = 0;
    let mut pcm = Vec::<f32>::with_capacity(1024);
    let mut wet = Vec::<f32>::with_capacity(1024);
    let mut drywet = 0.3;

    let mut builder = cubeb::StreamBuilder::<StereoFrame<f32>>::new();
    builder
        .name("mlr-rs")
        .default_output(&params)
        .latency(256)
        .data_callback(
            move |_input: &[StereoFrame<f32>], mut output: &mut [StereoFrame<f32>]| {
                match q2.pop() {
                    Ok(msg) => match msg {
                        Parameter::Absorbtion(v) => {
                            println!("set abs to {}", v);
                            reverb.set_absorbtion(v);
                        }
                        Parameter::Size(v) => {
                            println!("set size to {}", v);
                            reverb.set_size(v / 10.);
                        }
                        Parameter::Decay(v) => {
                            println!("set decay to {}", v);
                            reverb.set_decay(v);
                        }
                        Parameter::SaturationHardness(v) => {
                            println!("set hardness to {}", v);
                            reverb.set_hardness(v);
                        }
                        Parameter::Progression(v) => {
                            println!("set progression to {}", v);
                            reverb.set_progression(v);
                        }
                        Parameter::DryWet(v) => {
                            println!("set drywet to {}", v);
                            reverb.set_drywet(v);
                        }
                    },
                    Err(crossbeam::queue::PopError) => {}
                }
                let start = Instant::now();
                pcm.resize(output.len(), 0.0);
                wet.resize(output.len() * 2, 0.0);
                loop_player.extract(&mut pcm);
                reverb.process(&pcm, &mut wet);
                let mut output_idx = 0;
                for i in 0..pcm.len() {
                    output[i].l = wet[output_idx];
                    output_idx += 1;
                    output[i].r = wet[output_idx];
                    output_idx += 1;
                }
                let dd = start.elapsed();

                duration += dd.as_nanos();
                callback_count += 1;
                if PROFILE && callback_count % 50 == 0 {
                    let avg_duration_us = (duration as f32 / callback_count as f32) / 1000.;
                    let avg_duration_per_sample =
                        (duration as f32 / callback_count as f32) / 1000. / 512.;
                    let budget_us = 512. / 48000. * 1000. * 1000.;
                    println!(
                        "{}us ({} per sample, dsp load: {})",
                        avg_duration_us,
                        avg_duration_per_sample,
                        avg_duration_us / budget_us
                    );
                    callback_count = 0;
                    duration = 0;
                }
                output.len() as isize
            },
        )
        .state_callback(|state| {
            println!("stream {:?}", state);
        });

    let stream = builder.init(&ctx).expect("Failed to create cubeb stream");

    stream.start().unwrap();

    let mut led = [0.9, 0.3, 0.3, 0.5];

    match Monome::new("/prefix".to_string()) {
        Ok(mut monome) => {
            if monome.device_type() != MonomeDeviceType::Arc {
                ()
            }
            for i in 0..4 {
                monome.ring_all(i, 0);
                monome.ring_set(i, led[i] as u32, 15);
            }

            loop {
                loop {
                    match monome.poll() {
                        Some(MonomeEvent::EncoderDelta { n, delta }) => {
                            let n = n as usize;
                            monome.ring_set(n, led[n] as u32, 0);
                            led[n] = led[n] + (delta as f32 / 4.);
                            if led[n] < 8. {
                                led[n] = 8.;
                            }
                            if led[n] > 56. {
                                led[n] = 56.;
                            }
                            let msg = match n {
                                0 => Parameter::Absorbtion((led[n] - 8.) / 48. * 100.),
                                1 => Parameter::Size((led[n] - 8.) / 48. * 1000.),
                                2 => Parameter::Decay((led[n] - 8.) / 48. * 1.25),
                                3 => Parameter::DryWet((led[n] - 8.) / 48.),
                                _ => {
                                    panic!("ij");
                                }
                            };
                            q.push(msg).unwrap();
                            monome.ring_set(n, led[n] as u32, 15);
                        }
                        _ => {
                            break;
                        }
                    }
                }

                let refresh = time::Duration::from_millis(10);
                thread::sleep(refresh);
            }
        }
        _ => {}
    }

    loop {
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(len) =>  {
                let sp = buffer.trim().split(':');
                let v : Vec<&str> = sp.collect();
                if let Ok(idx) = v[0].parse::<u32>() {
                    if let Ok(val) = v[1].parse::<u32>() {
                        let msg = match idx {
                            0 => Parameter::Absorbtion(val as f32),
                            1 => Parameter::Size(val as f32),
                            2 => Parameter::Decay(val as f32 / 100.),
                            3 => Parameter::SaturationHardness(val as f32 / 10.),
                            4 => Parameter::Progression(1. + val as f32 / 10.),
                            5 => Parameter::DryWet(val as f32 / 100.),
                            _ => {
                                panic!("ij");
                            }
                        };
                        q.push(msg).unwrap();
                    }
                }

            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
}
