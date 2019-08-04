use byteorder::{LittleEndian, WriteBytesExt};
use std::fs::File;
use std::fs::*;
use std::io::prelude::*;

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
        b
    } else {
        a
    }
}

fn dump_wav(
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
                (&mut header[CHANNEL_OFFSET..])
                    .write_u16::<LittleEndian>(channel_count as u16)?;
                written += 2;
            }
            SAMPLE_RATE_OFFSET => {
                (&mut header[SAMPLE_RATE_OFFSET..])
                    .write_u32::<LittleEndian>(sample_rate)?;
                written += 4;
            }
            BLOCK_ALIGN_OFFSET => {
                (&mut header[BLOCK_ALIGN_OFFSET..])
                    .write_u16::<LittleEndian>((channel_count * 2) as u16)?;
                written += 2;
            }
            _ => {
                (&mut header[written..])
                    .write_u8(wav_header[written])?;
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
