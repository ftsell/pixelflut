use std::{simd::prelude::*, time};

use core::arch::x86_64::{__m128i, __m256i};

use crate::fast::{parse_hex_trick, parse_int_trick};

#[inline(always)]
pub fn parse_16_chars(chunk: Simd<u8, 16>) -> (u32, u32) {
    parse_16chars_m128i(chunk.into())
}

#[inline(always)]
pub fn parse_16_chars_hex(chunk: Simd<u8, 16>) -> u64 {
    use std::arch::x86_64::{
        _mm_cvtsi128_si64x, _mm_maddubs_epi16, _mm_set1_epi16, _mm_setr_epi8, _mm_shuffle_epi8,
    };
    let hexes = chunk & Simd::splat(0x40);
    let nibbles = (chunk | (hexes >> 3)) + (hexes >> 6);
    let nibbles = nibbles & Simd::splat(0x0f);

    let val: __m128i = nibbles.into();
    let val: __m128i = unsafe { _mm_maddubs_epi16(val, _mm_set1_epi16(0x0110)) };
    let shuf: __m128i = unsafe { _mm_setr_epi8(14, 12, 10, 8, 6, 4, 2, 0, -1, -1, -1, -1, -1, -1, -1, -1) };
    let val: __m128i = unsafe { _mm_shuffle_epi8(val, shuf) };

    let fin = unsafe { _mm_cvtsi128_si64x(val) };
    return fin as u64 & 0xffffff;
}

#[inline(always)]
pub fn parse_16chars_m128i(mut chunk: __m128i) -> (u32, u32) {
    use std::arch::x86_64::{
        _mm_and_si128, _mm_madd_epi16, _mm_maddubs_epi16, _mm_packus_epi32, _mm_set1_epi8, _mm_set_epi16,
        _mm_set_epi8,
    };

    chunk = unsafe {
        let mask = _mm_set1_epi8(0x0f);
        _mm_and_si128(chunk, mask)
    };

    // chunk is 16 bytes
    chunk = unsafe {
        let mult = _mm_set_epi8(1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10);
        _mm_maddubs_epi16(chunk, mult)
    };
    // chunk is 8 u16
    chunk = unsafe {
        let mult = _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100);
        _mm_madd_epi16(chunk, mult)
    };
    // chunk is 4 u32

    chunk = unsafe {
        let packed = _mm_packus_epi32(chunk, chunk);
        let mult = _mm_set_epi16(0, 0, 0, 0, 1, 10000, 1, 10000);
        _mm_madd_epi16(packed, mult)
    };
    // chunk is 2 u32 at end of 4xu32 reg
    let res: Simd<u32, 4> = chunk.into();
    (res[0], res[1])
}

pub fn read_request_lines_simd(buf: &[u8], mut setpx: impl FnMut(u16, u16, u32)) {
    use std::simd::prelude::*;
    let mut pos = 0;
    while pos < buf.len() - 32 {
        let chunk: Simd<u8, 32> = Simd::from_slice(&buf[pos..pos + 32]);
        //print_simd_str(chunk);
        let nl = simd_first_newline(chunk).unwrap();
        //println!("nl: {nl}");
        pos += nl + 1;
        let mask: Mask<i8, 32> = Mask::from_bitmask((u32::MAX << nl) as u64);
        let line = mask.select(Simd::splat(0), chunk);
        //print_simd_str(line);
        parse_simd_line(line, &mut setpx);
    }
}

pub fn simd_count_newlines(buf: &[u8]) -> usize {
    let mut pos = 0;
    let mut nl: usize = 0;
    while pos < buf.len() - 32 {
        let chunk: Simd<u8, 32> = Simd::from_slice(&buf[pos..pos + 32]);
        let nls = chunk.simd_eq(Simd::splat('\n' as u8)).to_bitmask();
        nl += nls.count_ones() as usize;
        pos += 32;
    }
    nl
}

pub fn read_request_lines_simd_staged(
    buf: &[u8],
    intermediate: &mut Vec<u8>,
    setpx: impl FnMut(u16, u16, u32),
) {
    let requests = simd_count_newlines(buf);
    intermediate.resize(requests * 32, 0);
    align_requests(buf, intermediate.as_mut());
    handle_aligned_requests(intermediate.as_slice(), setpx);
}

#[inline(always)]
pub fn align_requests(buf: &[u8], out: &mut [u8]) {
    let mut inpos = 0;
    let mut outpos = 0;
    while inpos < buf.len() - 32 {
        let chunk: Simd<u8, 32> =
            Simd::from_array(unsafe { buf.get_unchecked(inpos..inpos + 32).try_into().unwrap() });
        let nl = simd_first_newline(chunk).unwrap_or(31);
        inpos += nl + 1;
        let mask: Mask<i8, 32> = Mask::from_bitmask((u32::MAX << nl) as u64);
        let line = mask.select(Simd::splat(0), chunk);
        let aligned = align_simd_req_line(line);
        unsafe {
            out.get_unchecked_mut(outpos..outpos + 32)
                .copy_from_slice(aligned.as_array().as_slice());
        }
        outpos += 32;
    }
}

#[inline(always)]
pub fn handle_aligned_requests(buf: &[u8], mut setpx: impl FnMut(u16, u16, u32)) {
    for chunk in buf.chunks_exact(32) {
        let (x, y) = parse_16_chars(Simd::from_array(chunk[08..24].try_into().unwrap()));
        //let x = parse_int_trick(u64::from_be_bytes(chunk[08..16].try_into().unwrap()));
        //let y = parse_int_trick(u64::from_be_bytes(chunk[16..24].try_into().unwrap()));
        let col = parse_16_chars_hex(chunk[16..32].try_into().unwrap());
        //let col = parse_hex_trick(u64::from_be_bytes(chunk[24..32].try_into().unwrap()));

        setpx(x as u16, y as u16, col as u32);
    }
}

#[inline(always)]
fn align_simd_req_line(mut line: Simd<u8, 32>) -> Simd<u8, 32> {
    use std::simd::prelude::*;
    let mask = line.simd_gt(Simd::splat(' ' as u8));
    let tok_mask = mask.to_bitmask();
    line = mask.select(line, Simd::splat(0));
    const OOB: usize = 31;
    //println!("{tok_mask:b}");
    let aligned = match tok_mask {
        // HELP / SIZE
        0b1111 => line,
        #[rustfmt::skip]
        0b1111110101011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   3
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   5
            , OOB, OOB,   7,   8,   9,  10,  11,  12
            ]),

        #[rustfmt::skip]
        // PX XX Y CCCCCC
        0b11111101011011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB,   3,   4
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   6
            , OOB, OOB,   8,   9,  10,  11,  12,  13
            ]),
        #[rustfmt::skip]
        // PX XXX Y CCCCCC
        0b111111010111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB,   3,   4,   5
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   7
            , OOB, OOB,   9,  10,  11,  12,  13,  14
            ]),
        #[rustfmt::skip]
        // PX XXXX Y CCCCCC
        0b1111110101111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB,   3,   4,   5,   6
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   8
            , OOB, OOB,  10,  11,  12,  13,  14,  15
            ]),
        #[rustfmt::skip]
        // PX X YY CCCCCC
        0b11111101101011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   3
            , OOB, OOB, OOB, OOB, OOB, OOB,   5,   6
            , OOB, OOB,   8,   9,  10,  11,  12,  13
            ]),
        #[rustfmt::skip]
        // PX XX YY CCCCCC
        0b111111011011011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB,   3,   4
            , OOB, OOB, OOB, OOB, OOB, OOB,   6,   7
            , OOB, OOB,   9,  10,  11,  12,  13,  14
            ]),
        #[rustfmt::skip]
        // PX XXX YY CCCCCC
        0b1111110110111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB,   3,   4,   5
            , OOB, OOB, OOB, OOB, OOB, OOB,   7,   8
            , OOB, OOB,  10,  11,  12,  13,  14,  15
            ]),
        #[rustfmt::skip]
        // PX XXXX YY CCCCCC
        0b11111101101111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB,   3,   4,   5,   6
            , OOB, OOB, OOB, OOB, OOB, OOB,   8,   9
            , OOB, OOB,  11,  12,  13,  14,  15,  16
            ]),
        #[rustfmt::skip]
        // PX X YYY CCCCCC
        0b111111011101011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   3
            , OOB, OOB, OOB, OOB, OOB,   5,   6,   7
            , OOB, OOB,   9,  10,  11,  12,  13,  14
            ]),
        #[rustfmt::skip]
        // PX XX YYY CCCCCC
        0b1111110111011011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB,   3,   4
            , OOB, OOB, OOB, OOB, OOB,   6,   7,   8
            , OOB, OOB,  10,  11,  12,  13,  14,  15
            ]),
        #[rustfmt::skip]
        // PX XXX YYY CCCCCC
        0b11111101110111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB,   3,   4,   5
            , OOB, OOB, OOB, OOB, OOB,   7,   8,   9
            , OOB, OOB,  11,  12,  13,  14,  15,  16
            ]),
        #[rustfmt::skip]
        // PX XXXX YYY CCCCCC
        0b111111011101111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB,   3,   4,   5,   6
            , OOB, OOB, OOB, OOB, OOB,   8,   9,  10
            , OOB, OOB,  12,  13,  14,  15,  16,  17
            ]),
        #[rustfmt::skip]
        // PX X YYYY CCCCCC
        0b1111110111101011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   3
            , OOB, OOB, OOB, OOB,   5,   6,   7,   8
            , OOB, OOB,  10,  11,  12,  13,  14,  15
            ]),
        #[rustfmt::skip]
        // PX XX YYYY CCCCCC
        0b11111101111011011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB,   3,   4
            , OOB, OOB, OOB, OOB,   6,   7,   8,   9
            , OOB, OOB,  11,  12,  13,  14,  15,  16
            ]),
        #[rustfmt::skip]
        // PX XXX YYYY CCCCCC
        0b111111011110111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB,   3,   4,   5
            , OOB, OOB, OOB, OOB,   7,   8,   9,  10
            , OOB, OOB,  12,  13,  14,  15,  16,  17
            ]),
        #[rustfmt::skip]
        // PX XXXX YYYY CCCCCC
        0b1111110111101111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB,   3,   4,   5,   6
            , OOB, OOB, OOB, OOB,   8,   9,  10,  11
            , OOB, OOB,  13,  14,  15,  16,  17,  18
            ]),

        _ => panic!(),
    };
    aligned
}

#[inline(always)]
fn parse_simd_line(line: Simd<u8, 32>, mut setpx: impl FnMut(u16, u16, u32)) {
    use std::simd::prelude::*;
    let aligned = align_simd_req_line(line);
    let raw: __m256i = aligned.into();
    let u64s: Simd<u64, 4> = raw.into();
    let x = parse_int_trick(u64::from_be(u64s[1]));
    let y = parse_int_trick(u64::from_be(u64s[2]));
    let col = parse_hex_trick(u64::from_be(u64s[3]));

    setpx(x as u16, y as u16, col);
}

#[inline(always)]
pub fn simd_first_newline(simd: Simd<u8, 32>) -> Option<usize> {
    use std::simd::prelude::*;
    let newline: Mask<_, 32> = simd.simd_eq(Simd::splat('\n' as u8));
    newline.first_set()
}

pub fn print_simd_str(simd: Simd<u8, 32>) {
    let array = simd.to_array();
    let str = std::str::from_utf8(&array).unwrap();
    println!("{:?}", str);
}

#[cfg(test)]
mod tests {
    extern crate test;

    use test::Bencher;

    use crate::Pixmap;

    use super::*;

    #[test]
    fn test_parse_16_chars_hex() {
        for hex in [0x0, 0x1, 0x10, 0x25, 0x1234, 0xdadbef] {
            let mut buf: [u8; 16] = [' ' as u8; 16];
            let input = format!("{hex:06x}");
            buf[16 - input.len()..16].copy_from_slice(input.as_bytes());
            //println!("{}", std::str::from_utf8(&buf).unwrap());
            let parsed = parse_16_chars_hex(Simd::from_array(buf));
            //println!("{hex:x} {parsed:x}");
            assert_eq!(hex, parsed);
        }
    }

    #[test]
    fn test_align_simd_req_line() {
        let mut buf: [u8; 32] = [' ' as u8; 32];
        let input = "PX 1 4 ABCDEF\n";
        buf[0..input.len()].copy_from_slice(input.as_bytes());
        let simd: Simd<u8, 32> = Simd::from_array(buf);
        let aligned = align_simd_req_line(simd);
        println!("{aligned:?}");
    }

    #[test]
    fn test_parse_simd_line() {
        for x in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 23, 543, 1234] {
            for y in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 23, 543, 1234] {
                for col in [0xabcdef, 0x000000, 0x123456, 0xdedbef] {
                    let mut buf: [u8; 32] = [' ' as u8; 32];
                    let input = format!("PX {x} {y} {col:06x}\n");
                    //println!("{input:?}");

                    buf[0..input.len()].copy_from_slice(input.as_bytes());
                    let simd: Simd<u8, 32> = Simd::from_array(buf);

                    //let aligned = align_simd_req_line(simd);
                    //println!("{aligned:?}");
                    let mut px = 9999;
                    let mut py = 9999;
                    let mut pcol = 0x9999;
                    parse_simd_line(simd, |x, y, c| {
                        px = x;
                        py = y;
                        pcol = c;
                    });
                    assert_eq!(x, px, "x doesn't match");
                    assert_eq!(y, py, "y doesn't match");
                    assert_eq!(col, pcol, "col doesn't match {col:x} {pcol:x}");
                }
            }
        }
    }

    #[test]
    fn test_handle_aligned_requests() {
        let mut requests = Vec::new();
        let mut input = String::new();
        for x in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 23, 543, 1234] {
            for y in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 23, 543, 1234] {
                for col in [0xabcdef, 0x000000, 0x123456, 0xdedbef] {
                    requests.push((x, y, col));
                    let line = format!("PX {x} {y} {col:06x}\n");
                    input.extend(line.chars());
                }
            }
        }
        let mut buf = Vec::new();
        let aligned = {
            let requests = simd_count_newlines(input.as_bytes());
            buf.resize(requests * 32, 0);
            align_requests(input.as_bytes(), buf.as_mut());
            &buf[0..requests * 32]
        };

        let mut i = 0;
        handle_aligned_requests(aligned, |x, y, col| {
            let (rx, ry, rcol) = requests[i];
            assert_eq!(rx, x, "x not equal");
            assert_eq!(ry, y, "y not equal");
            assert_eq!(rcol, col, "col not equal");
            i += 1;
        });
    }

    #[bench]
    fn bench_align_simd_req_line(b: &mut Bencher) {
        let mut buf: [u8; 32] = [' ' as u8; 32];
        let input = "PX 1 4 ABCDEF\n";
        buf[0..input.len()].copy_from_slice(input.as_bytes());
        let simd: Simd<u8, 32> = Simd::from_array(buf);
        b.iter(|| align_simd_req_line(std::hint::black_box(simd)));
    }

    #[bench]
    fn bench_parse_req_line(b: &mut Bencher) {
        let mut buf: [u8; 32] = [' ' as u8; 32];
        let input = "PX 1 4 ABCDEF\n";
        buf[0..input.len()].copy_from_slice(input.as_bytes());
        let simd: Simd<u8, 32> = Simd::from_array(buf);
        let mut sum = 0;
        b.iter(|| {
            parse_simd_line(std::hint::black_box(simd), |a, b, c| {
                sum += a as usize;
                sum += b as usize;
                sum += c as usize;
            })
        });
        println!("{sum}");
    }

    #[test]
    fn test_read_request_line_simd() {
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        let buf = input.as_slice();
        let mut pixmap = Pixmap::new(1920, 1080);
        read_request_lines_simd(std::hint::black_box(buf), |x, y, c| {
            let idx = y as usize * pixmap.width as usize + x as usize;
            if let Some(px) = pixmap.pixels.get_mut(idx) {
                *px = c;
            }
        })
    }

    #[test]
    fn test_read_request_line_simd_staged() {
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        let buf = input.as_slice();
        let mut intemediate = Vec::new();
        let mut pixmap = Pixmap::new(1920, 1080);
        read_request_lines_simd_staged(std::hint::black_box(buf), &mut intemediate, |x, y, c| {
            let idx = y as usize * pixmap.width as usize + x as usize;
            if let Some(px) = pixmap.pixels.get_mut(idx) {
                *px = c;
            }
        });
        println!("sum: {}", pixmap.pixels.iter().map(|&x| x as u64).sum::<u64>());
    }
}
