use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use criterion_cycles_per_byte::CyclesPerByte;
use pixelparser::{align, compliant, fast::consume_nogen, simdimpl, Pixmap};

pub fn parse_fast(c: &mut Criterion) {
    let input = std::fs::read("testcase.txt").expect("no testcase file found");
    let mut pixmap = Pixmap::new(1920, 1080);
    let mut group = c.benchmark_group("parsing");
    let input = &input.as_slice()[0..1 << 13];
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("fast", |b| {
        b.iter(|| {
            consume_nogen(black_box(input), &mut pixmap);
        })
    });
    group.finish();
}

pub fn parse_simd(c: &mut Criterion) {
    let input = std::fs::read("testcase.txt").expect("no testcase file found");
    let mut pixmap = Pixmap::new(1920, 1080);
    let mut group = c.benchmark_group("parsing");
    let input = input.as_slice();
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("simd", |b| {
        b.iter(|| {
            simdimpl::read_request_lines_simd(criterion::black_box(input), |x, y, c| {
                let idx = y as usize * pixmap.width as usize + x as usize;
                if let Some(px) = pixmap.pixels.get_mut(idx) {
                    *px = c;
                }
            })
        })
    });
    group.finish();
}

pub fn parse_simd_staged(c: &mut Criterion) {
    let input = std::fs::read("testcase.txt").expect("no testcase file found");
    let mut pixmap = Pixmap::new(1920, 1080);
    let mut group = c.benchmark_group("parsing");
    let input = input.as_slice();
    let mut intermediate = Vec::with_capacity(input.len() * 32);
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("simd_staged", |b| {
        b.iter(|| {
            simdimpl::read_request_lines_simd_staged(
                criterion::black_box(input),
                &mut intermediate,
                |x, y, c| {
                    let idx = y as usize * pixmap.width as usize + x as usize;
                    if let Some(px) = pixmap.pixels.get_mut(idx) {
                        *px = c;
                    }
                },
            )
        })
    });
    group.finish();
}

pub fn parse_align(c: &mut Criterion) {
    let input = std::fs::read("testcase.txt").expect("no testcase file found");
    //let mut pixmap = Pixmap::new(1920, 1080);
    let mut group = c.benchmark_group("parsing");
    let input = &input.as_slice()[0..1 << 16];
    let mut buf = vec![0u64; input.len() * 4];
    //let mut output: Vec<Request> = Vec::with_capacity(buf.len() / 32);
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("align", |b| {
        b.iter(|| {
            buf.fill(0);
            align::align_input_u64(black_box(input), black_box(&mut buf));
        })
    });
    group.finish();
}

/*
pub fn parse_compliant(c: &mut Criterion) {
    let input = std::fs::read("testcase.txt").expect("no testcase file found");
    let mut pixmap = Pixmap::new(1920, 1080);
    let mut group = c.benchmark_group("parsing");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("compliant", |b| {
        b.iter(|| {
            compliant::consume(black_box(input.as_slice()), |x, y, c| {
                let idx = pixmap.width as usize * y as usize + x as usize;
                if let Some(px) = pixmap.pixels.get_mut(idx) {
                    *px = c;
                }
            })
        })
    });
    group.finish();
}
*/

pub fn write_pressure(c: &mut Criterion) {
    let mut pixmap = Pixmap::new(1920, 1080);

    let mut group = c.benchmark_group("write_pressure");
    group.throughput(Throughput::Bytes((pixmap.pixels.len() * 4) as u64));
    group.bench_function("consume", |b| {
        b.iter(|| {
            let mut x = 0;
            for px in pixmap.pixels.iter_mut() {
                *px += x as u32;
                x += 1;
            }
        });
    });
    group.finish();
}

/*
criterion_group!(
    name = benches;
    config = Criterion::default().with_measurement(CyclesPerByte);
    targets = parse_fast, parse_simd
);
*/

criterion_group!(
    benches,
    parse_fast,
    parse_simd,
    parse_simd_staged,
    parse_align,
    /*parse_compliant,*/ write_pressure
);
criterion_main!(benches);
