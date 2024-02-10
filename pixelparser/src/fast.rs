use std::{error::Error, io::BufRead};

#[inline(always)]
pub fn parse_int_trick(mut chunk: u64) -> u64 {
    chunk = u64::from_be(chunk);

    // 1-byte mask trick (works on 4 pairs of single digits)
    let mut lower_digits = (chunk & 0x0f000f000f000f00) >> 8;
    let mut upper_digits = (chunk & 0x000f000f000f000f) * 10;
    chunk = lower_digits + upper_digits;

    // 2-byte mask trick (works on 2 pairs of two digits)
    lower_digits = (chunk & 0x00ff000000ff0000) >> 16;
    upper_digits = (chunk & 0x000000ff000000ff) * 100;
    chunk = lower_digits + upper_digits;

    // 4-byte mask trick (works on pair of four digits)
    lower_digits = (chunk & 0x0000ffff00000000) >> 32;
    upper_digits = (chunk & 0x000000000000ffff) * 10000;
    chunk = lower_digits + upper_digits;

    return chunk;
}

#[inline(always)]
pub fn parse_hex_trick(mut chunk: u64) -> u32 {
    let hexes = chunk & 0x4040404040404040;
    let nibbles = (chunk | (hexes >> 3)) + (hexes >> 6);
    chunk = u64::from_be(nibbles);

    // 1-byte mask trick (works on 4 pairs of single digits)
    let mut lower_digits = (chunk & 0x0f000f000f000f00) >> 8;
    let mut upper_digits = (chunk & 0x000f000f000f000f) * 16;
    chunk = lower_digits + upper_digits;

    // 2-byte mask trick (works on 2 pairs of two digits)
    lower_digits = (chunk & 0x00ff000000ff0000) >> 16;
    upper_digits = (chunk & 0x000000ff000000ff) * 16 * 16;
    chunk = lower_digits + upper_digits;

    // 4-byte mask trick (works on pair of four digits)
    lower_digits = (chunk & 0x0000ffff00000000) >> 32;
    upper_digits = (chunk & 0x000000000000ffff) * 16 * 16 * 16 * 16;
    chunk = lower_digits + upper_digits;

    return chunk as u32;
}

#[allow(unused_assignments)]
pub fn consume(mut reader: impl BufRead, mut setpx: impl FnMut(u16, u16, u32)) -> Result<(), Box<dyn Error>> {
    // h3 is the command, which should be checked against
    #[allow(unused)]
    let mut h3 = 0u64;
    let mut h2 = 0u64;
    let mut h1 = 0u64;
    let mut h0 = 0u64;
    loop {
        let input = reader.fill_buf()?;
        if input.is_empty() {
            return Ok(());
        }
        for &c in input {
            let byte: u8 = c;
            let prev = h0;
            h0 = h0 << 8 | byte as u64;
            if byte == ' ' as u8 {
                h3 = h2;
                h2 = h1;
                h1 = prev;
                h0 = 0;
            }

            if byte == '\n' as u8 {
                let x = parse_int_trick(h2);
                let y = parse_int_trick(h1);
                let c = parse_hex_trick(prev);

                setpx(x as u16, y as u16, c);
                h0 = 0;
            }
        }
        let read = input.len();
        reader.consume(read);
    }
}

pub fn count_lines(buf: &[u8]) -> usize {
    let mut lines = 0;
    for &c in buf {
        lines += if c == '\n' as u8 { 1 } else { 0 };
    }
    lines
}

pub mod swar {
    fn print128(inp: u128) {
        let bytes = inp.to_le_bytes();
        println!("{:?}", std::str::from_utf8(&bytes).unwrap());
    }

    #[allow(unused)]
    pub fn parse_input_swar(inp: &[u8]) -> usize {
        let chunks = inp.array_chunks::<16>();
        let mut pos: u128 = 16;
        let mut last_chunk: u128 = 0;
        for chunk in chunks {
            let chunk: u128 = u128::from_le_bytes(*chunk);
            let posmask = (1 << (pos * 8) % 128) - 1;

            print128(posmask);
            print128(chunk);
            print128(chunk >> 8);
            todo!();
        }
        return inp.len();
    }
}

#[cfg(test)]
pub mod tests {
    extern crate test;
    use test::Bencher;

    use super::swar::parse_input_swar;
    use super::{count_lines, parse_hex_trick, parse_int_trick};

    #[ignore]
    #[test]
    fn test_parse_input_swar() {
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        parse_input_swar(input.as_slice());
    }

    #[test]
    fn test_parse_hex_trick() {
        for i in 1..100920 {
            let input = format!("{i:0>8x}");
            let bytes: [u8; 8] = input.as_bytes().try_into().unwrap();
            let inp = u64::from_be_bytes(bytes);
            let res = parse_hex_trick(inp);
            assert_eq!(res, i, "{input:?}");
        }

        for i in 1..100920 {
            let input = format!("{i: >8x}");
            let mut bytes: [u8; 8] = input.as_bytes().try_into().unwrap();
            for b in bytes.iter_mut() {
                if *b == ' ' as u8 {
                    *b = 0;
                }
            }
            let input = u64::from_be_bytes(bytes);
            let res = parse_hex_trick(input);
            assert_eq!(res, i);
        }
    }

    #[test]
    fn test_parse_int_trick() {
        for i in 1..100920 {
            let input = format!("{i:0>8}");
            let bytes: [u8; 8] = input.as_bytes().try_into().unwrap();
            let input = u64::from_be_bytes(bytes);
            let res = parse_int_trick(input);
            assert_eq!(res, i);
        }

        for i in 1..100920 {
            let input = format!("{i: >8}");
            let mut bytes: [u8; 8] = input.as_bytes().try_into().unwrap();
            for b in bytes.iter_mut() {
                if *b == ' ' as u8 {
                    *b = 0;
                }
            }
            let input = u64::from_be_bytes(bytes);
            let res = parse_int_trick(input);
            assert_eq!(res, i);
        }
    }

    #[bench]
    fn bench_parse_int_trick(b: &mut Bencher) {
        let mut n = 0;
        b.iter(|| {
            n += 1;
            let inp = std::hint::black_box(n);
            parse_int_trick(inp)
        })
    }

    #[bench]
    fn bench_parse_int_trick_double(b: &mut Bencher) {
        let mut n = 0;
        let mut m = 1243;
        b.iter(|| {
            n += 1;
            m += 1;
            let inp = std::hint::black_box(n << 32 | m);
            parse_int_trick(inp)
        })
    }

    #[bench]
    fn bench_parse_hex_trick(b: &mut Bencher) {
        let mut n = 0;
        b.iter(|| {
            n += 1;
            let inp = std::hint::black_box(n);
            parse_hex_trick(inp)
        })
    }

    #[ignore]
    #[bench]
    fn bench_memory_throughput_single_char(b: &mut Bencher) {
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        b.iter(|| {
            let input = std::hint::black_box(input.as_slice());
            count_lines(input)
        });
    }
}
