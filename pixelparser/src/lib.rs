#![feature(test)]
#![feature(array_chunks)]
#![feature(portable_simd)]

use std::{
    error::Error,
    io::{BufRead, Read},
};

pub mod compliant;
pub mod fast;

pub mod align;
//wip
pub mod simdimpl;

struct CyclicRead<'b> {
    cursor: usize,
    bytes: &'b [u8],
}

impl<'b> Read for CyclicRead<'b> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remaining = &self.bytes[self.cursor..];
        if buf.len() >= remaining.len() {
            self.cursor = 0;
            buf[..remaining.len()].copy_from_slice(remaining);
            Ok(remaining.len())
        } else {
            buf.copy_from_slice(&remaining[0..buf.len()]);
            self.cursor += buf.len();
            Ok(buf.len())
        }
    }
}

type PxResult<T> = Result<T, Box<dyn Error>>;

pub type Pixel = u32;

pub struct Pixmap {
    pub pixels: Vec<Pixel>,
    pub width: u32,
    pub height: u32,
}

impl Pixmap {
    pub fn new(width: u32, height: u32) -> Self {
        Pixmap {
            pixels: vec![0u32; (width * height) as usize],
            width,
            height,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Request {
    SetPixel { x: u16, y: u16, color: Pixel },
    GetPixel { x: u16, y: u16 },
    GetSize,
    Help,
}

trait Consumer {
    fn consume(
        &mut self,
        reader: impl BufRead,
        setpx: impl FnMut(u16, u16, u32),
    ) -> Result<(), Box<dyn Error>>;
}

pub struct FastConsumer;
impl Consumer for FastConsumer {
    fn consume(
        &mut self,
        reader: impl BufRead,
        setpx: impl FnMut(u16, u16, u32),
    ) -> Result<(), Box<dyn Error>> {
        fast::consume(reader, setpx)
    }
}

pub struct CompliantConsumer;
impl Consumer for CompliantConsumer {
    fn consume(
        &mut self,
        reader: impl BufRead,
        setpx: impl FnMut(u16, u16, u32),
    ) -> Result<(), Box<dyn Error>> {
        compliant::consume(reader, setpx)
    }
}
