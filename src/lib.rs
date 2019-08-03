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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let paths = read_dir("samples").unwrap();

        let mut samples: Vec<Sample> = Vec::new();

        let mut file = File::create("out.pcm").unwrap();

        for path in paths {
            let path = path.unwrap();
            samples.push(Sample::new(&path));
        }

        let s = &samples[0];
        let mut d = DelayLine::new(44100);
        d.set_duration(2 * 128);
        let mut output_pcm = Vec::with_capacity(s.frames());

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
