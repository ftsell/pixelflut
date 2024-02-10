use std::simd::prelude::*;

use core::arch::x86_64::__m256i;

pub fn read_request_lines_simd(buf: &[u8], mut setpx: impl FnMut(u16, u16, u32)) {
    use std::simd::prelude::*;

    const SIZE: usize = 32;

    let idx: Simd<u8, SIZE> = Simd::from_array(IDX_32);
    let mut lines = 0;
    let mut s = 0;
    let mut chunks = buf.chunks_exact(SIZE);

    let mut last_chunk: Simd<u8, SIZE> = Simd::splat(' ' as u8);
    last_chunk[SIZE - 1] = '\n' as u8;
    let mut pos: u8 = SIZE as u8;

    for chunk in &mut chunks {
        let chunk: Simd<u8, SIZE> = Simd::from_slice(chunk);
        // change this while to if for performance
        while pos < SIZE as u8 {
            let mut line: Simd<u8, SIZE> = idx.simd_ge(Simd::splat(pos as u8)).select(last_chunk, chunk);

            // these two lines do the same thing, but one is very inefficient
            line = simd32_rotate_left_dyn(line, pos as u8);
            //line = line.swizzle_dyn((SIMD_IDX + Simd::splat(pos)) % Simd::splat(SIZE as u8));

            let nl_mask: Mask<i8, SIZE> = line.simd_eq(Simd::splat('\n' as u8));
            let first_newline = nl_mask.to_bitmask().trailing_zeros() as u8;

            line = idx
                .simd_le(Simd::splat(first_newline))
                .select(line, Simd::splat(0));

            let small: Simd<u8, 16> = unsafe {
                let arr = line.rotate_elements_left::<3>().to_array();
                let view: [[u8; 16]; 2] = std::mem::transmute(arr);
                Simd::from_array(view[0])
            };

            s += small.reduce_sum() as usize;

            pos += first_newline + 1;
            lines += 1;
        }
        last_chunk = chunk;
        pos %= SIZE as u8;
    }

    println!("s: {:?}", s);

    /*
    for &c in chunks.remainder() {
        let id_char = c > '_' as u8;
        tokens += if id_char && lastbit == 0 { 1 } else { 0 };
        lastbit = if id_char { 1 } else { 0 };
    }
    */
}

#[inline(always)]
fn simd32_rotate_left_dyn(simd: std::simd::Simd<u8, 32>, r: u8) -> std::simd::Simd<u8, 32> {
    match r % 32 {
        00 => simd.rotate_elements_left::<00>(),
        01 => simd.rotate_elements_left::<01>(),
        02 => simd.rotate_elements_left::<02>(),
        03 => simd.rotate_elements_left::<03>(),
        04 => simd.rotate_elements_left::<04>(),
        05 => simd.rotate_elements_left::<05>(),
        06 => simd.rotate_elements_left::<06>(),
        07 => simd.rotate_elements_left::<07>(),
        08 => simd.rotate_elements_left::<08>(),
        09 => simd.rotate_elements_left::<09>(),
        10 => simd.rotate_elements_left::<10>(),
        11 => simd.rotate_elements_left::<11>(),
        12 => simd.rotate_elements_left::<12>(),
        13 => simd.rotate_elements_left::<13>(),
        14 => simd.rotate_elements_left::<14>(),
        15 => simd.rotate_elements_left::<15>(),
        16 => simd.rotate_elements_left::<16>(),
        17 => simd.rotate_elements_left::<17>(),
        18 => simd.rotate_elements_left::<18>(),
        19 => simd.rotate_elements_left::<19>(),
        20 => simd.rotate_elements_left::<20>(),
        21 => simd.rotate_elements_left::<21>(),
        22 => simd.rotate_elements_left::<22>(),
        23 => simd.rotate_elements_left::<23>(),
        24 => simd.rotate_elements_left::<24>(),
        25 => simd.rotate_elements_left::<25>(),
        26 => simd.rotate_elements_left::<26>(),
        27 => simd.rotate_elements_left::<27>(),
        28 => simd.rotate_elements_left::<28>(),
        29 => simd.rotate_elements_left::<29>(),
        30 => simd.rotate_elements_left::<30>(),
        31 => simd.rotate_elements_left::<31>(),
        _ => unreachable!(),
    }
}

#[inline(always)]
fn simd64_rotate_left_dyn(simd: std::simd::Simd<u8, 64>, r: u8) -> std::simd::Simd<u8, 64> {
    match r % 64 {
        00 => simd.rotate_elements_left::<00>(),
        01 => simd.rotate_elements_left::<01>(),
        02 => simd.rotate_elements_left::<02>(),
        03 => simd.rotate_elements_left::<03>(),
        04 => simd.rotate_elements_left::<04>(),
        05 => simd.rotate_elements_left::<05>(),
        06 => simd.rotate_elements_left::<06>(),
        07 => simd.rotate_elements_left::<07>(),
        08 => simd.rotate_elements_left::<08>(),
        09 => simd.rotate_elements_left::<09>(),
        10 => simd.rotate_elements_left::<10>(),
        11 => simd.rotate_elements_left::<11>(),
        12 => simd.rotate_elements_left::<12>(),
        13 => simd.rotate_elements_left::<13>(),
        14 => simd.rotate_elements_left::<14>(),
        15 => simd.rotate_elements_left::<15>(),
        16 => simd.rotate_elements_left::<16>(),
        17 => simd.rotate_elements_left::<17>(),
        18 => simd.rotate_elements_left::<18>(),
        19 => simd.rotate_elements_left::<19>(),
        20 => simd.rotate_elements_left::<20>(),
        21 => simd.rotate_elements_left::<21>(),
        22 => simd.rotate_elements_left::<22>(),
        23 => simd.rotate_elements_left::<23>(),
        24 => simd.rotate_elements_left::<24>(),
        25 => simd.rotate_elements_left::<25>(),
        26 => simd.rotate_elements_left::<26>(),
        27 => simd.rotate_elements_left::<27>(),
        28 => simd.rotate_elements_left::<28>(),
        29 => simd.rotate_elements_left::<29>(),
        30 => simd.rotate_elements_left::<30>(),
        31 => simd.rotate_elements_left::<31>(),
        32 => simd.rotate_elements_left::<32>(),
        33 => simd.rotate_elements_left::<33>(),
        34 => simd.rotate_elements_left::<34>(),
        35 => simd.rotate_elements_left::<35>(),
        36 => simd.rotate_elements_left::<36>(),
        37 => simd.rotate_elements_left::<37>(),
        38 => simd.rotate_elements_left::<38>(),
        39 => simd.rotate_elements_left::<39>(),
        40 => simd.rotate_elements_left::<40>(),
        41 => simd.rotate_elements_left::<41>(),
        42 => simd.rotate_elements_left::<42>(),
        43 => simd.rotate_elements_left::<43>(),
        44 => simd.rotate_elements_left::<44>(),
        45 => simd.rotate_elements_left::<45>(),
        46 => simd.rotate_elements_left::<46>(),
        47 => simd.rotate_elements_left::<47>(),
        48 => simd.rotate_elements_left::<48>(),
        49 => simd.rotate_elements_left::<49>(),
        50 => simd.rotate_elements_left::<50>(),
        51 => simd.rotate_elements_left::<51>(),
        52 => simd.rotate_elements_left::<52>(),
        53 => simd.rotate_elements_left::<53>(),
        54 => simd.rotate_elements_left::<54>(),
        55 => simd.rotate_elements_left::<55>(),
        56 => simd.rotate_elements_left::<56>(),
        57 => simd.rotate_elements_left::<57>(),
        58 => simd.rotate_elements_left::<58>(),
        59 => simd.rotate_elements_left::<59>(),
        60 => simd.rotate_elements_left::<60>(),
        61 => simd.rotate_elements_left::<61>(),
        62 => simd.rotate_elements_left::<62>(),
        63 => simd.rotate_elements_left::<63>(),
        _ => unreachable!(),
    }
}

const IDX_32: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
    29, 30, 31,
];

const IDX_64: [u8; 64] = [
    00, 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51,
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
];

#[inline(always)]
pub fn parse_hex(inp: u64) -> u32 {
    let alpha = 0x404040404040 & inp;
    let low = 0x0f0f0f0f0f0f & inp;
    let nibbles = low | (alpha >> 3) + (alpha >> 6);
    //println!("{nibbles:0b}");
    let x0y0z = (nibbles | nibbles >> 4) & 0xff00ff00ff;
    //println!("{x0y0z:0b}");
    let xyz = (x0y0z & 0xff00000000) >> 16 | (x0y0z & 0xff0000) >> 8 | (x0y0z & 0xff);
    //println!("{xyz:0b}");
    xyz as u32
}

#[inline(always)]
fn parse_simd_line(line: Simd<u8, 32>, setpx: &mut impl FnMut(u16, u16, u32)) {
    //print_simd_str(line);
    use std::simd::prelude::*;
    let tok_mask = line.simd_gt(Simd::splat(' ' as u8)).to_bitmask();
    let arr = line.to_array();

    match tok_mask {
        // HELP / SIZE
        0b1111 => {}
        // PX X Y CCCCCC
        0b1111110101011 => {
            let x = arr[3] - '0' as u8;
            let y = arr[5] - '0' as u8;
            let c = parse_hex(
                arr[7] as u64
                    | (arr[8] as u64) << 8
                    | (arr[9] as u64) << 16
                    | (arr[10] as u64) << 24
                    | (arr[11] as u64) << 32
                    | (arr[12] as u64) << 40,
            );
            setpx(x as u16, y as u16, c);
        }
        // PX XX Y CCCCCC
        0b11111101011011 => {}
        // PX XXX Y CCCCCC
        0b111111010111011 => {}
        // PX XXXX Y CCCCCC
        0b1111110101111011 => {}

        // PX X YY CCCCCC
        0b11111101101011 => {}
        // PX XX YY CCCCCC
        0b111111011011011 => {}
        // PX XXX YY CCCCCC
        0b1111110110111011 => {}
        // PX XXXX YY CCCCCC
        0b11111101101111011 => {}

        // PX X YYY CCCCCC
        0b111111011101011 => {}
        // PX XX YYY CCCCCC
        0b1111110111011011 => {}
        // PX XXX YYY CCCCCC
        0b11111101110111011 => {}
        // PX XXXX YYY CCCCCC
        0b111111011101111011 => {}

        // PX X YYYY CCCCCC
        0b1111110111101011 => {}
        // PX XX YYYY CCCCCC
        0b11111101111011011 => {}
        // PX XXX YYYY CCCCCC
        0b111111011110111011 => {}
        // PX XXXX YYYY CCCCCC
        0b1111110111101111011 => {}

        _ => {}
    }
}

#[inline(always)]
pub fn simd_first_newline(simd: Simd<u8, 32>) -> Option<usize> {
    use std::simd::prelude::*;
    let SIMD_NL: Simd<u8, 32> = Simd::splat('\n' as u8);
    let newline: Mask<_, 32> = simd.simd_eq(SIMD_NL);
    Some((newline.to_bitmask() as u32).trailing_zeros() as usize)
    //let bitmask = newline.to_bitmask();
    //let newline = bitmask64_first_set_idx(bitmask);
    //33 - newline
}

#[inline(always)]
pub fn simd_parse_slice(slice: &[u8]) -> (Simd<u8, 32>, Option<usize>) {
    let simd = Simd::from_slice(&slice[0..32]);
    let newline = simd_first_newline(simd);
    (simd, newline)
}

pub fn parse_simd() {
    let mut buf: [u8; 32] = [' ' as u8; 32];
    let input = "   PX 123   456 ABCDE\n";
    buf[0..input.len()].copy_from_slice(input.as_bytes());
    let simd: Simd<u8, 32> = Simd::from_array(buf);
    let spaces: Simd<i8, 32> = simd.simd_eq(Simd::splat(' ' as u8)).to_int();
    let left_spaces = spaces.rotate_elements_left::<1>();
    let end = (left_spaces - spaces).simd_eq(Simd::splat(-1));
    let start = (left_spaces - spaces)
        .rotate_elements_right::<1>()
        .simd_eq(Simd::splat(1));

    println!("{:3?}", simd);
    println!("{:3?}", start.to_int());
    println!("{:3?}", end.to_int());
}

const IDX: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
    29, 30, 31,
];
const SIMD_IDX: Simd<u8, 32> = Simd::from_array(IDX);

pub fn print_simd_str(simd: Simd<u8, 32>) {
    let array = simd.to_array();
    let str = std::str::from_utf8(&array).unwrap();
    println!("{:?}", str);
}

pub fn print_simd_str64(simd: Simd<u8, 64>) {
    let array = simd.to_array();
    let str = std::str::from_utf8(&array).unwrap();
    println!("{:?}", str);
}

#[inline(always)]
fn select_simd(input: Simd<u8, 32>, start: u8, end: u8, target: u8) -> Simd<u8, 32> {
    use std::simd::prelude::*;
    let idx = SIMD_IDX - Simd::splat(target - end + 1);
    let mask = idx.simd_ge(Simd::splat(start)) & (idx.simd_lt(Simd::splat(end)));
    let idx = mask.select(idx, Simd::splat(255));
    let arg1 = input.swizzle_dyn(idx);
    arg1
}

#[inline(always)]
fn simd_align(input: Simd<u8, 32>, select: Mask<i8, 32>) -> Simd<u8, 32> {
    use std::simd::prelude::*;
    let spaces = Simd::splat(' ' as u8);
    let masked = select.select(input, spaces);
    let token = masked.simd_ne(spaces).to_bitmask() as u32;
    let mut pos = 0;
    let mut starts = [0; 4];
    let mut ends = [0; 4];
    for i in 0..4 {
        pos += (token >> pos).trailing_zeros();
        starts[i] = pos;
        pos += (token >> pos).trailing_ones();
        ends[i] = pos;
    }
    let arg0 = select_simd(masked, starts[0] as u8, ends[0] as u8, ends[0] as u8 - 1);
    let arg1 = select_simd(masked, starts[1] as u8, ends[1] as u8, 8 + 7);
    let arg2 = select_simd(masked, starts[2] as u8, ends[2] as u8, 16 + 7);
    let arg3 = select_simd(masked, starts[3] as u8, ends[3] as u8, 24 + 7);
    let swizzled = arg0 | arg1 | arg2 | arg3;
    swizzled
}

#[inline(always)]
fn pull_simd(buf: &mut &[u8]) -> Simd<u8, 32> {
    use std::simd::prelude::*;
    let simd = Simd::from_slice(&buf[0..32]);
    let newline = simd_first_newline(simd).unwrap() as usize;
    *buf = &(*buf)[newline + 1..];
    let select = SIMD_IDX.simd_lt(Simd::splat(newline as u8));
    let aligned = simd_align(simd, select);
    aligned
}

type Line = Simd<u8, 32>;
type Segment = Simd<u64, 4>;

#[inline(always)]
fn simd_line_to_segments(line: Line) -> Segment {
    let x: __m256i = line.into();
    x.into()
}

#[inline(always)]
fn simd_segments_to_line(segment: Segment) -> Line {
    let x: __m256i = segment.into();
    x.into()
}

#[inline(always)]
fn simd_transpose(lines: [Line; 4]) -> (Line, Line, Line, Line) {
    let a = simd_line_to_segments(lines[0]);
    let b = simd_line_to_segments(lines[1]);
    let c = simd_line_to_segments(lines[2]);
    let d = simd_line_to_segments(lines[3]);

    // this changes the order of ops...
    let (px1, yc1) = a.interleave(b);
    let (px2, yc2) = c.interleave(d);
    let (p, x) = px1.interleave(px2);
    let (y, c) = yc1.interleave(yc2);

    (
        simd_segments_to_line(p),
        simd_segments_to_line(x),
        simd_segments_to_line(y),
        simd_segments_to_line(c),
    )
}

fn simd_parse_int(line: Line) -> Segment {
    //print_simd_str(line);
    use std::simd::prelude::*;
    let zeros: Simd<u8, 32> = Simd::splat(0);
    let digits = line.simd_ne(zeros);
    // byte coded decimal
    let bcd = line - Simd::splat('0' as u8);
    let bcd = digits.select(bcd, zeros);
    let mask: Mask<i8, 32> = Mask::from_bitmask(0x5555555555555555);
    let high = mask.select(bcd, zeros);
    let bcd = bcd.rotate_elements_left::<1>() + high * Simd::splat(10);
    let bcd = mask.select(bcd, zeros);
    let bcd: __m256i = bcd.into();
    let bcd: Simd<u16, 16> = bcd.into();

    let mask: Mask<i16, 16> = Mask::from_bitmask(0x55555555);
    let zeros: Simd<u16, 16> = Simd::splat(0);
    let high = mask.select(bcd, zeros);
    // maybe this should be * 1000?
    let bcd = bcd.rotate_elements_left::<1>() + high * Simd::splat(100);
    let bcd = mask.select(bcd, zeros);
    let bcd: __m256i = bcd.into();
    let bcd: Simd<u32, 8> = bcd.into();

    let mask: Mask<i32, 8> = Mask::from_bitmask(0x5555);
    let zeros: Simd<u32, 8> = Simd::splat(0);
    let high = mask.select(bcd, zeros);
    // maybe this should be * 1000?
    let bcd = bcd.rotate_elements_left::<1>() + high * Simd::splat(1000);
    let bcd = mask.select(bcd, zeros);
    let bcd: __m256i = bcd.into();
    let bcd: Simd<u64, 4> = bcd.into();

    bcd
}

pub fn parse_simd_bunched(buf: &mut &[u8]) -> (u64, u64, u64) {
    use std::simd::prelude::*;
    assert!(buf.len() > 32 * 4);
    let (cmd, x, y, color) = simd_transpose([pull_simd(buf), pull_simd(buf), pull_simd(buf), pull_simd(buf)]);

    let x = simd_parse_int(x);
    let y = simd_parse_int(y);
    let color = simd_parse_int(color);
    (x.reduce_sum(), y.reduce_sum(), color.reduce_sum())
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;

    #[test]
    fn test_simd_newlines() {
        for i in 0..32 {
            let mut test: Simd<u8, 32> = Simd::splat(0);
            test[i] = '\n' as u8;
            println!("test: {:?}", test);
            let first_newline = simd_first_newline(test);
            assert_eq!(i, first_newline.unwrap() as usize);
        }

        for i in 0..32 {
            let mut test: Simd<u8, 32> = Simd::splat(0);
            test[31] = '\n' as u8;
            test[i] = '\n' as u8;
            println!("test: {:?}", test);
            let first_newline = simd_first_newline(test);
            assert_eq!(i, first_newline.unwrap() as usize);
        }
    }
}
