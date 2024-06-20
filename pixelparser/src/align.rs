use std::{cmp, process::Command};

use crate::{
    fast::{parse_hex_trick, parse_int_trick},
    Request,
};

pub fn align_input_u64(input: &[u8], output: &mut [u64]) -> (usize, usize) {
    let mut out = 0;
    let mut inp = 0;
    let mut cur: u64 = 0;
    while (inp < input.len()) & (out < output.len()) {
        let c = input[inp];
        inp += 1;
        let prev = cur;
        cur = (cur << 8) | c as u64;
        if c <= ' ' as u8 {
            output[out] = prev;
            out += 1;
            if c == '\n' as u8 {
                out += 3;
                out &= usize::MAX << 2;
            }
        }
    }
    (inp, out)
}

fn print_u64_chunk(inp: u64) {
    let bytes = inp.to_ne_bytes();
    println!("{:?} {bytes:x?}", std::str::from_utf8(&bytes));
}

fn parse_aligned(input: &[u64], output: &mut Vec<Request>) {
    const PX: u64 = u64::from_be_bytes([0, 0, 0, 0, 0, 0, 'P' as u8, 'X' as u8]);
    assert!(input.len() % 4 == 0);
    for chunk in input.chunks_exact(4) {
        //println!("{chunk:x?}");
        let command = chunk[0];
        let arg1 = chunk[1];
        let arg2 = chunk[2];
        let arg3 = chunk[3];
        //print_u64_chunk(command);
        //print_u64_chunk(arg1);
        //print_u64_chunk(arg2);
        //print_u64_chunk(arg3);
        let arg1 = parse_int_trick(arg1);
        let arg2 = parse_int_trick(arg2);
        let arg3 = parse_hex_trick(arg3);
        //println!("{arg1} {arg2} {arg3:x}");
        let command = match command {
            PX => Request::SetPixel {
                x: arg1 as u16,
                y: arg2 as u16,
                color: arg3,
            },
            _ => continue,
        };
        //println!("{:?}", command);
        output.push(command);
        //todo!()
    }
}

pub fn consume(input: &[u8], aligned: &mut [u64], output: &mut Vec<Request>) {
    aligned.fill(0);
    output.clear();
    let _ = align_input_u64(input, aligned);
    parse_aligned(&aligned, output);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_input() {
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        let input = &input[0..1 << 12];
        let mut output = vec![0u64; input.len() * 4];
        align_input_u64(input, &mut output);
    }

    #[test]
    fn test_parse_aligned() {
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        let input = &input[0..1 << 12];
        let mut output = vec![0u64; input.len() * 4];
        let (inp_read, out_wrote) = align_input_u64(input, &mut output);
        parse_aligned(&output[0..out_wrote & (usize::MAX << 2)], &mut Vec::new());
    }
}
