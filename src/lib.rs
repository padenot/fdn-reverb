use byteorder::{WriteBytesExt, LittleEndian};
use audrey::*;
use log::*;
use std::fs::*;
use std::fs::File;
use std::io::prelude::*;
use std::fs::DirEntry;
use std::vec;
use std::ops::Index;
use std::mem::size_of;
use std::f32::consts::PI;

const BLOCK_SIZE: usize = 32;

fn clamp<T>(v: T, lower_bound: T, higher_bound: T) -> T
where T: std::cmp::PartialOrd {
    if v < lower_bound {
        return lower_bound;
    } else if v > higher_bound {
        return higher_bound;
    } else {
        return v;
    }
}

fn max<T>(a: T, b: T) -> T 
where T: std::cmp::PartialOrd {
    if a > b {
        b
    } else {
        a
    }
}

fn dump_wav(file_name: &str, samples: &[i16], channel_count: u32, sample_rate: u32) -> Result<(), std::io::Error> {
    const COUNT: usize = 44;
    let wav_header = [
        // RIFF header
        0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45,
        // fmt chunk. We always write 16-bit samples.
        0x66, 0x6d, 0x74, 0x20, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x10, 0x00,
        // data chunk
        0x64, 0x61, 0x74, 0x61, 0xFE, 0xFF, 0xFF, 0x7F];
    const CHANNEL_OFFSET: usize = 22;
    const SAMPLE_RATE_OFFSET:usize = 24;
    const BLOCK_ALIGN_OFFSET:usize = 32;
    let mut header = [0 as u8; COUNT];
    let mut written = 0;

    println!("size {}", wav_header.len());

    while written != COUNT {
        match written {
            CHANNEL_OFFSET => {
                (&mut header[CHANNEL_OFFSET..]).write_u16::<LittleEndian>(channel_count as u16).unwrap();
                written += 2;
            }
            SAMPLE_RATE_OFFSET => {
                (&mut header[SAMPLE_RATE_OFFSET..]).write_u32::<LittleEndian>(sample_rate).unwrap();
                written += 4;
            }
            BLOCK_ALIGN_OFFSET => {
                (&mut header[BLOCK_ALIGN_OFFSET..]).write_u16::<LittleEndian>((channel_count * 2) as u16).unwrap();
                written += 2;
            }
            _ => {
                (&mut header[written..]).write_u8(wav_header[written]).unwrap();
                written += 1;
            }
        }
    }

    let mut file = File::create(file_name).unwrap();
    file.write_all(&header).unwrap();
    for i in samples.iter() {
        file.write_i16::<LittleEndian>(*i).unwrap();
    }
    Ok(())
}

struct Sample {
    name: String,
    channels: u32,
    rate: u32,
    data: Vec<f32>,
}

impl Sample {
    fn new(path: &DirEntry) -> Sample {
        info!("Loading {:?}...", path.path());
        let mut file = audrey::read::open(&path.path()).unwrap();
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
    fn channels(&self) -> u32 {
        self.channels
    }
    fn frames(&self) -> usize {
        self.data.len() / self.channels as usize
    }
    fn duration(&self) -> f32 {
        (self.data.len() as f32) / self.channels as f32 / self.rate as f32
    }
    fn rate(&self) -> u32 {
        self.rate
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn slice(&self, start: usize, size: usize) -> &[f32] {
        let mut real_size = size;
        if start + size >= self.data.len() {
            real_size = self.data.len() - start;
        }
        &self.data[start..start+real_size]
    }
}


impl Index<usize> for Sample {
    type Output = f32;

    fn index(&self, index: usize) -> &f32 {
        &self.data[index]
    }
}

struct DelayLine {
    memory:  Vec<f32>,
    duration: usize,
    read_index: usize,
    write_index:  usize
}

impl DelayLine {
    fn new(max_duration: usize) -> DelayLine {
        let mut v = Vec::<f32>::with_capacity(max_duration);
        v.resize(max_duration, 0.0);
        let mut d = DelayLine {
            memory: v,
            duration: 0,
            read_index: 0,
            write_index: 0
        };
        d.set_duration(max_duration);
        return d;
    }
    fn set_duration(&mut self, duration: usize) {
        self.duration = duration;
        self.write_index = self.write_index % duration;
        self.read_index = if self.write_index < self.duration {
            self.memory.len() - (duration - self.write_index)
        } else {
            self.write_index - duration
        };
    }
    fn write(&mut self, input: &[f32]) {
        let mut w = self.write_index;
        let l = self.memory.len();
        for i in input.iter() {
            self.memory[w] = *i;
            w = (w + 1) % l;
        }
        self.write_index = w;
    }
    fn read(&mut self, output: &mut [f32]) {
        let mut r = self.read_index;
        let l = self.memory.len();
        for o in output.iter_mut() {
            *o = self.memory[r];
            r = (r + 1) % l;
        }
        self.read_index = r;
    }
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        self.write(input);
        self.read(output);
        for it in input.iter().zip(output.iter_mut()) {
            let (inp,out) = it;
            *out += inp * 0.5;
        }
    }
}

enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    LowShelf,
    HighShelf,
    Peaking,
    AllPass,
    Notch
}

struct Biquad {
    nyquist: f32,
    filter_type: FilterType,
    // Coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,

    // Memory
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

// Based on the web audio api implem: https://webaudio.github.io/web-audio-api/#biquadfilternode
impl Biquad {
    fn new(filter_type: FilterType, sample_rate: f32) -> Biquad {
        Biquad {
            nyquist: sample_rate / 2.0,
            filter_type,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
    fn low_pass(frequency: f32, q: f32, sample_rate: f32) -> Biquad {
        let nyquist = sample_rate / 2.0;
        let normalized_freq = frequency / nyquist;
        let mut b = Biquad::new(FilterType::LowPass, sample_rate);
        b.set_low_pass_params(normalized_freq, q);
        return b;
    }
    fn set_low_pass_params(&mut self, cutoff: f32, resonance: f32) {
        let clamped_cutoff = clamp(cutoff, 0., 1.);

        if  clamped_cutoff == 1.  {
            // When cutoff is 1, the z-transform is 1.
            self.set_normalized_coefficients(1., 0., 0., 1., 0., 0.);
        } else if cutoff > 0. {
            // Compute biquad coefficients for lowpass filter
            let clamped_resonance = max(0.0, resonance);  // can't go negative
            let g = 10.0f32.powf(-0.05 * clamped_resonance);
            let w0 = PI * cutoff;
            let cos_w0 = w0.cos();
            let alpha = 0.5 * w0.sin() * g;

            let b1 = 1.0 - cos_w0;
            let b0 = 0.5 * b1;
            let b2 = b0;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cos_w0;
            let a2 = 1.0 - alpha;

            self.set_normalized_coefficients(b0, b1, b2, a0, a1, a2);
        } else {
            // When cutoff is zero, nothing gets through the filter, so set
            // coefficients up correctly.
            self.set_normalized_coefficients(0., 0., 0., 1., 0., 0.);
        }
    }
    fn high_pass(frequency: f32, q: f32, sample_rate: f32) -> Biquad {
        let nyquist = sample_rate / 2.0;
        let normalized_freq = frequency / nyquist;
        let mut b = Biquad::new(FilterType::HighPass, sample_rate);
        b.set_highpass_params(normalized_freq, q);
        return b;
    }
    fn set_highpass_params(&mut self, cutoff: f32, resonance: f32) {
        // Limit cutoff to 0 to 1.
        let clamped_cutoff = clamp(cutoff, 0., 1.0);

        if clamped_cutoff == 1. {
            // The z-transform is 0.
            self.set_normalized_coefficients(0., 0., 0., 1., 0., 0.);
        } else if clamped_cutoff > 0. {
            // Compute biquad coefficients for highpass filter
            let clamped_resonance = max(0.0, resonance);  // can't go negative
            let g = 10.0f32.powf(-0.05 * clamped_resonance);
            let w0 = PI * cutoff;
            let cos_w0 = w0.cos();
            let alpha = 0.5 * w0.sin() * g;

            let b1 = -1.0 - cos_w0;
            let b0 = -0.5 * b1;
            let b2 = b0;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cos_w0;
            let a2 = 1.0 - alpha;

            self.set_normalized_coefficients(b0, b1, b2, a0, a1, a2);
        } else {
            // When cutoff is zero, we need to be careful because the above
            // gives a quadratic divided by the same quadratic, with poles
            // and zeros on the unit circle in the same place. When cutoff
            // is zero, the z-transform is 1.
            self.set_normalized_coefficients(1., 0., 0., 1., 0., 0.);
        }
    }

    fn set_low_shelf_params(&mut self, frequency: f32, db_gain: f32) {
        let clamped_frequency = clamp(frequency, 0., 1.);

        let a = 10.0f32.powf(db_gain / 40.);

        if clamped_frequency == 1. {
            // The z-transform is a constant gain.
            self.set_normalized_coefficients(a * a, 0., 0., 1., 0., 0.);
        } else if clamped_frequency > 0. {
            let w0 = PI * clamped_frequency;
            let s = 1.;  // filter slope (1 is max value)
            let alpha = 0.5 * w0.sin() * ((a + 1. / a) * (1. / s - 1.) + 2.).sqrt();
            let k = w0.cos();
            let k2 = 2. * a.sqrt() * alpha;
            let a_plus_one = a + 1.;
            let a_minus_one = a - 1.;

            let b0 = a * (a_plus_one - a_minus_one * k + k2);
            let b1 = 2. * a * (a_minus_one - a_plus_one * k);
            let b2 = a * (a_plus_one - a_minus_one * k - k2);
            let a0 = a_plus_one + a_minus_one * k + k2;
            let a1 = -2. * (a_minus_one + a_plus_one * k);
            let a2 = a_plus_one + a_minus_one * k - k2;

            self.set_normalized_coefficients(b0, b1, b2, a0, a1, a2);
        } else {
            // When frequency is 0, the z-transform is 1.
            self.set_normalized_coefficients(1., 0., 0., 1., 0., 0.);
        }
    }

    fn set_high_shelf_params(&mut self, frequency: f32, db_gain: f32) {
        // Clip frequencies to between 0 and 1, inclusive.
        let clamped_frequency = clamp(frequency, 0.0, 1.0);

        let a = 10.0f32.powf(db_gain / 40.);

        if clamped_frequency == 1. {
            // The z-transform is 1.
            self.set_normalized_coefficients(1., 0., 0., 1., 0., 0.);
        } else if clamped_frequency > 0. {
            let w0 = PI * frequency;
            let s = 1.;  // filter slope (1 is max value)
            let alpha = 0.5 * w0.sin() * ((a + 1. / a) * (1. / s - 1.) + 2.).sqrt();
            let k = w0.cos();
            let k2 = 2. * a.sqrt() * alpha;
            let a_plus_one = a + 1.;
            let a_minus_one = a - 1.;

            let b0 = a * (a_plus_one + a_minus_one * k + k2);
            let b1 = -2. * a * (a_minus_one + a_plus_one * k);
            let b2 = a * (a_plus_one + a_minus_one * k - k2);
            let a0 = a_plus_one - a_minus_one * k + k2;
            let a1 = 2. * (a_minus_one - a_plus_one * k);
            let a2 = a_plus_one - a_minus_one * k - k2;

            self.set_normalized_coefficients(b0, b1, b2, a0, a1, a2);
        } else {
            // When frequency = 0, the filter is just a gain, a^2.
            self.set_normalized_coefficients(a * a, 0., 0., 1., 0., 0.);
        }
    }

    fn set_peaking_params(&mut self, frequency: f32, q: f32, db_gain: f32) {
        // Clip frequencies to between 0 and 1, inclusive.
        let clamped_frequency = clamp(frequency, 0.0, 1.0);

        // Don't let q go negative, which causes an unstable filter.
        let clamped_q = max(0.0, q);

        let a = 10.0f32.powf(db_gain / 40.);

        if clamped_frequency > 0. && clamped_frequency < 1. {
            if clamped_q > 0. {
                let w0 = PI * clamped_frequency;
                let alpha = w0.sin() / (2. * q);
                let k = w0.cos();

                let b0 = 1. + alpha * a;
                let b1 = -2. * k;
                let b2 = 1. - alpha * a;
                let a0 = 1. + alpha / a;
                let a1 = -2. * k;
                let a2 = 1. - alpha / a;

                self.set_normalized_coefficients(b0, b1, b2, a0, a1, a2);
            } else {
                // When q = 0, the above formulas have problems. If we look at
                // the z-transform, we can see that the limit as q->0 is a^2, so
                // set the filter that way.
                self.set_normalized_coefficients(a * a, 0., 0., 1., 0., 0.);
            }
        } else {
            // When frequency is 0 or 1, the z-transform is 1.
            self.set_normalized_coefficients(1., 0., 0., 1., 0., 0.);
        }
    }

    fn set_allpass_params(&mut self, frequency: f32, q: f32) {
        let clamped_frequency = clamp(frequency, 0.0, 1.0);

        let clamped_q = max(0.0, q);

        if clamped_frequency > 0. && clamped_frequency < 1. {
            if clamped_q > 0. {
                let w0 = PI * clamped_frequency;
                let alpha = w0.sin() / (2. * clamped_q);
                let k = w0.cos();

                let b0 = 1. - alpha;
                let b1 = -2. * k;
                let b2 = 1. + alpha;
                let a0 = 1. + alpha;
                let a1 = -2. * k;
                let a2 = 1. - alpha;

                self.set_normalized_coefficients(b0, b1, b2, a0, a1, a2);
            } else {
                // When q = 0, the above formulas have problems. If we look at
                // the z-transform, we can see that the limit as q->0 is -1, so
                // set the filter that way.
                self.set_normalized_coefficients(-1., 0., 0., 1., 0., 0.);
            }
        } else {
            // When frequency is 0 or 1, the z-transform is 1.
            self.set_normalized_coefficients(1., 0., 0., 1., 0., 0.);
        }
    }

    fn set_notch_params(&mut self, frequency: f32, q: f32) {
        let clamped_frequency = clamp(frequency, 0.0, 1.0);

        let clamped_q = max(0.0, q);

        if clamped_frequency > 0. && clamped_frequency < 1. {
            if clamped_q > 0. {
                let w0 = PI * clamped_frequency;
                let alpha = w0.sin() / (2. * clamped_q);
                let k = w0.cos();

                let b0 = 1.;
                let b1 = -2. * k;
                let b2 = 1.;
                let a0 = 1. + alpha;
                let a1 = -2. * k;
                let a2 = 1. - alpha;

                self.set_normalized_coefficients(b0, b1, b2, a0, a1, a2);
            } else {
                // When q = 0, the above formulas have problems. If we look at
                // the z-transform, we can see that the limit as q->0 is 0, so
                // set the filter that way.
                self.set_normalized_coefficients(0., 0., 0., 1., 0., 0.);
            }
        } else {
            // When frequency is 0 or 1, the z-transform is 1.
            self.set_normalized_coefficients(1., 0., 0., 1., 0., 0.);
        }
    }

    fn set_bandpass_params(&mut self, frequency: f32, q: f32) {
        let clamped_frequency = max(0.0, frequency);

        let clamped_q = max(0.0, q);

        if clamped_frequency > 0. && clamped_frequency < 1. {
            let w0 = PI * clamped_frequency;
            if clamped_q > 0. {
                let alpha = w0.sin() / (2. * clamped_q);
                let k = w0.cos();

                let b0 = alpha;
                let b1 = 0.;
                let b2 = -alpha;
                let a0 = 1. + alpha;
                let a1 = -2. * k;
                let a2 = 1. - alpha;

                self.set_normalized_coefficients(b0, b1, b2, a0, a1, a2);
            } else {
                // When q = 0, the above formulas have problems. If we look at
                // the z-transform, we can see that the limit as q->0 is 1, so
                // set the filter that way.
                self.set_normalized_coefficients(1., 0., 0., 1., 0., 0.);
            }
        } else {
            // When the cutoff is zero, the z-transform approaches 0, if q
            // > 0. When both q and cutoff are zero, the z-transform is
            // pretty much undefined. What should we do in this case?
            // For now, just make the filter 0. When the cutoff is 1, the
            // z-transform also approaches 0.
            self.set_normalized_coefficients(0., 0., 0., 1., 0., 0.);
        }
    }
    fn set_normalized_coefficients(&mut self,
                                   b0: f32, b1: f32, b2: f32,
                                   a0: f32, a1: f32, a2: f32) {
        let a0_inverse = 1. / a0;

        self.b0 = b0 * a0_inverse;
        self.b1 = b1 * a0_inverse;
        self.b2 = b2 * a0_inverse;
        self.a1 = a1 * a0_inverse;
        self.a2 = a2 * a0_inverse;
    }
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for it in input.iter().zip(output.iter_mut()) {
            let (i,o) = it;
            *o = self.b0 * *i + self.b1 * self.x1 + self.b2 * self.x2
                - self.a1 * self.y1 - self.a2 * self.y2;
            self.x2 = self.x1;
            self.x1 = *i;
            self.y2 = self.y1;
            self.y1 = *o;

        }
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
        let paths = read_dir("samples").unwrap();

        let mut samples: Vec<Sample> = Vec::new();

        for path in paths {
            let path = path.unwrap();
            samples.push(Sample::new(&path));
        }

        let s = &samples[0];
        //let mut d = DelayLine::new(44100);
        //d.set_duration(2 * 128);
        let mut d = Biquad::low_pass(300., 2., 44100.);
        let mut output_pcm = Vec::<i16>::with_capacity(s.frames());
        output_pcm.resize(s.frames(), 0);

        let mut i: usize = 0;
        let mut output = Vec::<f32>::with_capacity(BLOCK_SIZE);
        let mut j = 0;

        loop {
            let input = s.slice(i, BLOCK_SIZE);
            i += input.len();
            if input.len() == 0 {
                break;
            }
            output.resize(input.len(), 0.);
            d.process(&input, &mut output);
            for i in output.iter() {
                // clip and convert to 16bits
                let clipped;
                if *i > 1.0 {
                    clipped  = 1.0;
                } else if *i < -1.0 {
                    clipped = -1.0;
                } else {
                    clipped = *i;
                }
                let sample: i16 = (clipped * (2 << 14) as f32) as i16;
                output_pcm[j] = sample;
                j+=1;
            }
        }
        dump_wav("out.wav", &output_pcm, s.channels(), s.rate()).unwrap();
    }
}
