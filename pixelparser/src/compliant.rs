use std::{error::Error, fmt::Display, io::BufRead};

#[allow(unused_imports)]
use crate::Pixmap;
use crate::{PxResult, Request};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ParseErr {
    UnknownCommand,
    ExpectedToken,
}

impl Display for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErr::UnknownCommand => write!(f, "Unknown Command"),
            ParseErr::ExpectedToken => write!(f, "Expected Token"),
        }
    }
}
impl Error for ParseErr {}

#[inline(always)]
fn parse_set_pixel(x: &str, y: &str, px: &str) -> Result<Request, ParseErr> {
    let xres = atoi_simd::parse_pos::<u16>(x.as_bytes());
    let yres = atoi_simd::parse_pos::<u16>(y.as_bytes());
    let cres = u32::from_str_radix(px, 16);
    match (xres, yres, cres) {
        (Ok(x), Ok(y), Ok(color)) => Ok(Request::SetPixel { x, y, color }),
        (_, _, _) => Err(ParseErr::UnknownCommand),
    }
}

#[inline(always)]
fn parse_get_pixel(x: &str, y: &str) -> Result<Request, ParseErr> {
    let xres = atoi_simd::parse_pos::<u16>(x.as_bytes());
    let yres = atoi_simd::parse_pos::<u16>(y.as_bytes());
    match (xres, yres) {
        (Ok(x), Ok(y)) => Ok(Request::GetPixel { x, y }),
        (_, _) => Err(ParseErr::UnknownCommand),
    }
}

pub struct TokBuf<'s> {
    toks: [Option<&'s str>; 4],
    len: usize,
}

impl<'s> TokBuf<'s> {
    #[inline(always)]
    fn tokens(&self) -> &[&'s str] {
        unsafe { std::mem::transmute(&self.toks[0..self.len]) }
    }
}

impl<'s> FromIterator<&'s str> for TokBuf<'s> {
    #[inline(always)]
    fn from_iter<T: IntoIterator<Item = &'s str>>(iter: T) -> Self {
        let mut this = Self {
            toks: [None, None, None, None],
            len: 0,
        };
        let mut iter = iter.into_iter();
        for i in 0..4 {
            if let Some(t) = iter.next() {
                this.toks[i] = Some(t);
                this.len = i + 1;
            } else {
                break;
            }
        }
        this
    }
}

#[inline(always)]
pub fn parse_request_line(line: &str) -> Result<Request, ParseErr> {
    let toks: TokBuf<'_> = line.split_whitespace().collect();
    let toks = toks.tokens();
    match toks.len() {
        4 => parse_set_pixel(toks[1], toks[2], toks[3]),
        3 => parse_get_pixel(toks[1], toks[2]),
        2 => Err(ParseErr::UnknownCommand),
        1 => match toks[0] {
            "SIZE" => Ok(Request::GetSize),
            "HELP" => Ok(Request::Help),
            _ => Err(ParseErr::UnknownCommand),
        },
        0 => Err(ParseErr::ExpectedToken),
        _ => unreachable!(),
    }
}

#[cfg(test)]
pub fn read_request_slice<'b, 'r: 'b>(
    read: &'r mut impl BufRead,
    buf: &'b mut Vec<u8>,
) -> std::io::Result<Option<&'b str>> {
    use std::io::Read;
    // Clear previous line, if any, but keep capacity
    buf.clear();

    // This constant is important, because otherwise
    // a malicious client could exhaust our memory by never sending
    // a newline.
    const MAX_LINE_LENGTH: usize = 32;
    let read = read.take(MAX_LINE_LENGTH as u64).read_until('\n' as u8, buf)?;
    use std::io;
    match read {
        0 => Ok(None),
        MAX_LINE_LENGTH => Err(io::Error::new(io::ErrorKind::Other, "MAX_LINE_LENGTH exceeded")),
        _ => {
            let line = &buf[0..read];
            if line.is_ascii() {
                let str = unsafe { std::str::from_utf8_unchecked(line) };
                Ok(Some(str))
            } else {
                Err(io::Error::new(io::ErrorKind::Other, "not an ascii string"))
            }
        }
    }
}

#[inline(always)]
pub fn read_request_line<'l, 'r: 'l>(
    read: &'r mut impl BufRead,
    line: &'l mut String,
) -> std::io::Result<Option<&'l str>> {
    use std::io::Read;
    // Clear previous line, if any, but keep capacity
    line.clear();

    // This constant is important, because otherwise
    // a malicious client could exhaust our memory by never sending
    // a newline.
    const MAX_LINE_LENGTH: usize = 32;
    let read = read.take(MAX_LINE_LENGTH as u64).read_line(line)?;
    use std::io;
    match read {
        0 => Ok(None),
        MAX_LINE_LENGTH => Err(io::Error::new(io::ErrorKind::Other, "MAX_LINE_LENGTH exceeded")),
        _ => Ok(Some(line.as_str())),
    }
}

#[cfg(test)]
pub fn pixmap_consumer(mut read: impl BufRead, pixmap: &mut Pixmap) -> PxResult<()> {
    let mut line = String::with_capacity(32);
    while let Some(line) = read_request_line(&mut read, &mut line)? {
        let req = parse_request_line(line)?;
        if let Request::SetPixel { x, y, color } = req {
            let idx = x as u32 + y as u32 * pixmap.width;
            *pixmap.pixels.get_mut(idx as usize).unwrap() = color;
        };
    }
    Ok(())
}

pub fn consume(mut read: impl BufRead, mut setpx: impl FnMut(u16, u16, u32)) -> Result<(), Box<dyn Error>> {
    let mut line = String::with_capacity(32);
    while let Some(line) = read_request_line(&mut read, &mut line)? {
        let req = parse_request_line(line)?;
        if let Request::SetPixel { x, y, color } = req {
            setpx(x, y, color)
        };
    }
    Ok(())
}

#[allow(dead_code)]
fn handle_requests(
    mut read: impl BufRead,
    mut on_request: impl FnMut(Request),
) -> Result<(), Box<dyn Error>> {
    let mut line = String::with_capacity(32);
    while let Some(line) = read_request_line(&mut read, &mut line)? {
        let req = parse_request_line(line)?;
        on_request(req)
    }
    Ok(())
}

#[allow(dead_code)]
pub fn print_consumer(read: impl BufRead) -> PxResult<()> {
    handle_requests(read, |req| println!("{:?}", req))
}

#[cfg(test)]
mod tests {
    extern crate test;
    use std::io::BufReader;

    use crate::CyclicRead;

    use super::*;
    use std::hint::black_box;
    use test::bench::Bencher;

    #[test]
    fn test_parse_px() {
        fn test_case(line: &str, res: Request) {
            let req = parse_request_line(line);
            assert_eq!(req, Ok(res));
        }
        use Request::*;
        test_case("HELP", Help);
        test_case("SIZE", GetSize);
        test_case("PX 0 0 0", SetPixel { x: 0, y: 0, color: 0 });
        test_case(
            "PX 12 34 56",
            SetPixel {
                x: 12,
                y: 34,
                color: 5 * 16 + 6,
            },
        );
    }

    #[bench]
    fn bench_parse_get_pixel(b: &mut Bencher) {
        let x = black_box("17");
        let y = black_box("7632");
        b.iter(move || parse_get_pixel(x, y).unwrap());
    }

    #[bench]
    fn bench_parse_set_pixel(b: &mut Bencher) {
        let x = black_box("17");
        let y = black_box("7632");
        let px = black_box("57A011");
        b.iter(move || parse_set_pixel(x, y, px).unwrap());
    }

    #[bench]
    fn bench_line_split_whitespace(b: &mut Bencher) {
        let input = black_box("PX 12345 27890 ffDE12");
        b.iter(|| input.split_whitespace().last());
    }

    #[bench]
    fn bench_parse_line(b: &mut Bencher) {
        let input = black_box("PX  12345 27890 ffDE12");
        b.iter(move || parse_request_line(&input).unwrap())
    }

    #[bench]
    fn bench_read_line(b: &mut Bencher) {
        let mut line = String::with_capacity(32);
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        let mut input = BufReader::new(CyclicRead {
            cursor: 0,
            bytes: input.as_slice(),
        });
        b.iter(move || read_request_line(&mut input, &mut line).unwrap().is_some())
    }

    #[bench]
    fn bench_read_line_ascii(b: &mut Bencher) {
        let mut line = Vec::with_capacity(32);
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        let mut input = BufReader::new(CyclicRead {
            cursor: 0,
            bytes: input.as_slice(),
        });
        b.iter(move || read_request_slice(&mut input, &mut line).unwrap().is_some())
    }

    #[bench]
    fn bench_parse_lines(b: &mut Bencher) {
        let mut line = String::with_capacity(32);
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        let mut input = BufReader::new(CyclicRead {
            cursor: 0,
            bytes: input.as_slice(),
        });
        b.iter(move || {
            let line = read_request_line(&mut input, &mut line).unwrap().unwrap();
            let req = parse_request_line(line).expect("should be valid request");
            req
        })
    }

    #[bench]
    fn bench_parse_lines_ascii(b: &mut Bencher) {
        let mut line = Vec::with_capacity(32);
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        let mut input = BufReader::new(CyclicRead {
            cursor: 0,
            bytes: input.as_slice(),
        });
        b.iter(move || {
            let line = read_request_slice(&mut input, &mut line).unwrap().unwrap();
            let req = parse_request_line(line).expect("should be valid request");
            req
        })
    }

    #[bench]
    fn bench_pixmap_requests(b: &mut Bencher) {
        let mut pixmap = Pixmap::new(1920, 1080);
        let mut line = String::with_capacity(32);
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        let mut input = BufReader::new(CyclicRead {
            cursor: 0,
            bytes: input.as_slice(),
        });
        let pixref = &mut pixmap;
        b.iter(move || {
            let line = read_request_line(&mut input, &mut line).unwrap().unwrap();
            let req = parse_request_line(line).expect("should be valid request");
            let Request::SetPixel { x, y, color } = req else {
                panic!("not a set pixel request")
            };
            let idx = x as u32 + y as u32 * pixref.width;
            *pixref.pixels.get_mut(idx as usize).unwrap() = color;
        });
        let sum: usize = pixmap.pixels.iter().map(|&x| x as usize).sum();
        println!("sum: {}", sum);
    }

    #[bench]
    fn bench_pixmap_requests_ascii(b: &mut Bencher) {
        let mut pixmap = Pixmap::new(1920, 1080);
        let mut line = Vec::with_capacity(32);
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        let mut input = BufReader::new(CyclicRead {
            cursor: 0,
            bytes: input.as_slice(),
        });
        let pixref = &mut pixmap;
        b.iter(move || {
            let line = read_request_slice(&mut input, &mut line).unwrap().unwrap();
            let req = parse_request_line(line).expect("should be valid request");
            let Request::SetPixel { x, y, color } = req else {
                panic!("not a set pixel request")
            };
            let idx = x as u32 + y as u32 * pixref.width;
            *pixref.pixels.get_mut(idx as usize).unwrap() = color;
        });
        let sum: usize = pixmap.pixels.iter().map(|&x| x as usize).sum();
        println!("sum: {}", sum);
    }

    #[allow(dead_code)]
    //#[bench]
    fn bench_pixmap_consumer(b: &mut Bencher) {
        let mut pixmap = Pixmap::new(1920, 1080);
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        let pixref = &mut pixmap;
        b.iter(move || {
            let input = BufReader::new(std::io::Cursor::new(input.as_slice()));
            pixmap_consumer(input, pixref).unwrap();
        });
        let sum: usize = pixmap.pixels.iter().map(|&x| x as usize).sum();
        println!("sum: {}", sum);
    }
}
