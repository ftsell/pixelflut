use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use pixelparser::{compliant, fast::consume, Pixmap};

pub fn parse_fast(c: &mut Criterion) {
    let input = std::fs::read("testcase.txt").expect("no testcase file found");
    let mut pixmap = Pixmap::new(1920, 1080);
    let mut group = c.benchmark_group("parsing");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("fast", |b| {
        b.iter(|| {
            consume(black_box(input.as_slice()), |x, y, c| {
                let idx = pixmap.width as usize * y as usize + x as usize;
                if let Some(px) = pixmap.pixels.get_mut(idx) {
                    *px = c;
                }
            })
        })
    });
    group.finish();
}

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

criterion_group!(benches, parse_fast, parse_compliant, write_pressure);
criterion_main!(benches);
