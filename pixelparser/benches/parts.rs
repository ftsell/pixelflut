use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use pixelparser::{
    fast::{count_lines, parse_int_trick},
    simdimpl::{align_requests, handle_aligned_requests, simd_count_newlines},
    Pixmap,
};

pub fn bench_count_lines(c: &mut Criterion) {
    let input = std::fs::read("testcase.txt").expect("no testcase file found");
    let mut group = c.benchmark_group("read_pressure");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("count_lines", |b| {
        b.iter(|| count_lines(black_box(input.as_slice())))
    });
}

pub fn bench_parse_int(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_pressure");
    group.throughput(Throughput::Bytes(4 as u64));
    let i: u32 = 00192837;
    let input = format!("{i:0>8x}");
    let bytes: [u8; 8] = input.as_bytes().try_into().unwrap();
    let inp = u64::from_be_bytes(bytes);
    group.bench_function("parse_int", |b| b.iter(|| parse_int_trick(black_box(inp))));
}

pub fn bench_parse_double_int(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_pressure");
    group.throughput(Throughput::Bytes(4 as u64));
    let i: u32 = 00192837;
    let input = format!("{i:0>8x}");
    let bytes: [u8; 8] = input.as_bytes().try_into().unwrap();
    let inp1 = u64::from_be_bytes(bytes);
    let i: u32 = 00192837;
    let input = format!("{i:0>8x}");
    let bytes: [u8; 8] = input.as_bytes().try_into().unwrap();
    let inp2 = u64::from_be_bytes(bytes);
    group.bench_function("parse_double_int", |b| {
        b.iter(|| parse_int_trick(black_box(inp1) << 32 | black_box(inp2)))
    });
}

pub fn bench_handle_aligned_data(c: &mut Criterion) {
    let mut pixmap = Pixmap::new(1920, 1080);
    let input = std::fs::read("testcase.txt").expect("no testcase file found");
    let mut buf = Vec::new();
    let aligned = {
        let requests = simd_count_newlines(&input);
        buf.resize(requests * 32, 0);
        align_requests(&input, buf.as_mut());
        &buf[0..requests * 32]
    };

    let mut group = c.benchmark_group("simd");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("handle_aligned_data", |b| {
        b.iter(|| {
            handle_aligned_requests(criterion::black_box(aligned), |x, y, col| {
                let idx = y as usize * pixmap.width as usize + x as usize;
                if let Some(px) = pixmap.pixels.get_mut(idx) {
                    *px = col;
                }
            })
        })
    });

    let sum: usize = pixmap.pixels.iter().map(|&x| x as usize).sum();
    println!("checksum: {sum}");
}

criterion_group!(
    benches,
    bench_count_lines,
    bench_parse_int,
    bench_parse_double_int,
    bench_handle_aligned_data,
);
criterion_main!(benches);
