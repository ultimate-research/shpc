use criterion::{black_box, criterion_group, criterion_main, Criterion};

use shpc::sh::{compress_coefficients, decompress_coefficients};

pub fn compress_coefficients_benchmark(c: &mut Criterion) {
    c.bench_function("compress_coefficients", |b| {
        b.iter(|| {
            compress_coefficients(
                black_box(1.0),
                black_box(2.0),
                black_box([1.0, 2.0, 3.0, 4.0]),
            )
        })
    });
}

pub fn decompress_coefficients_benchmark(c: &mut Criterion) {
    c.bench_function("decompress_coefficients", |b| {
        b.iter(|| {
            decompress_coefficients(
                black_box(1.0),
                black_box(2.0),
                black_box([128, 255, 37, 64]),
            )
        })
    });
}

criterion_group!(
    benches,
    compress_coefficients_benchmark,
    decompress_coefficients_benchmark
);
criterion_main!(benches);
