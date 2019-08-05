use audrey::*;
use byteorder::{LittleEndian, WriteBytesExt};
use log::*;
use std::fs::DirEntry;
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::ops::Index;

pub fn clamp<T>(v: T, lower_bound: T, higher_bound: T) -> T
where
    T: std::cmp::PartialOrd,
{
    if v < lower_bound {
        return lower_bound;
    } else if v > higher_bound {
        return higher_bound;
    } else {
        return v;
    }
}

pub fn max<T>(a: T, b: T) -> T
where
    T: std::cmp::PartialOrd,
{
    if a > b {
        a
    } else {
        b
    }
}

pub fn dump_wav(
    file_name: &str,
    samples: &[i16],
    channel_count: u32,
    sample_rate: u32,
) -> Result<(), std::io::Error> {
    const COUNT: usize = 44;
    let wav_header = [
        // RIFF header
        0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45,
        // fmt chunk. We always write 16-bit samples.
        0x66, 0x6d, 0x74, 0x20, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x10, 0x00, // data chunk
        0x64, 0x61, 0x74, 0x61, 0xFE, 0xFF, 0xFF, 0x7F,
    ];
    const CHANNEL_OFFSET: usize = 22;
    const SAMPLE_RATE_OFFSET: usize = 24;
    const BLOCK_ALIGN_OFFSET: usize = 32;
    let mut header = [0 as u8; COUNT];
    let mut written = 0;

    while written != COUNT {
        match written {
            CHANNEL_OFFSET => {
                (&mut header[CHANNEL_OFFSET..]).write_u16::<LittleEndian>(channel_count as u16)?;
                written += 2;
            }
            SAMPLE_RATE_OFFSET => {
                (&mut header[SAMPLE_RATE_OFFSET..]).write_u32::<LittleEndian>(sample_rate)?;
                written += 4;
            }
            BLOCK_ALIGN_OFFSET => {
                (&mut header[BLOCK_ALIGN_OFFSET..])
                    .write_u16::<LittleEndian>((channel_count * 2) as u16)?;
                written += 2;
            }
            _ => {
                (&mut header[written..]).write_u8(wav_header[written])?;
                written += 1;
            }
        }
    }

    let mut file = File::create(file_name)?;
    file.write_all(&header)?;
    for i in samples.iter() {
        file.write_i16::<LittleEndian>(*i)?;
    }
    Ok(())
}

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

pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    if a < b {
        mem::swap(&mut a, &mut b);
    }

    while b != 0 {
        mem::swap(&mut a, &mut b);
        b %= a;
    }

    a
}

pub fn coprime(a: u64, b: u64) -> bool {
    gcd(a, b) == 1
}

pub fn coprime_with_series(proposed: u64, series: &[u64]) -> bool {
    for i in series.iter() {
        if !coprime(*i, proposed) {
            return false;
        }
    }

    true
}

/// Find a series of `count` number that are set coprime, and start at `start`, with a geometric
/// progression of ratio `factor`
pub fn coprime_with_progression(start: u64, factor: f32, count: usize) -> Vec<u64> {
    let mut series = Vec::with_capacity(count);
    let mut current = (start as f32 * factor) as u64;

    series.push(start);

    while series.len() != count {
        if coprime_with_series(current, &series) {
            series.push(current);
            continue;
        }
        while !coprime_with_series(current, &series) {
            current += 1;
        }
        series.push(current);
        current = (current as f32 * factor) as u64;
    }
    return series;
}
