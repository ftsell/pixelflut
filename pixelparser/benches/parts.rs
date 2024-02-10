use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use pixelparser::fast::{count_lines, parse_int_trick};

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

criterion_group!(
    benches,
    bench_count_lines,
    bench_parse_int,
    bench_parse_double_int
);
criterion_main!(benches);
