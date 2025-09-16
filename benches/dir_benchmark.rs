use criterion::{Criterion, criterion_group, criterion_main};
use std::path::Path;
use venv_rs_lib::dir_size::{Chonk, IterativeReader, ParallelReader, RecursiveReader};

fn bench_recursive(c: &mut Criterion) {
    let path = Path::new(".");
    let method = RecursiveReader;

    c.bench_function("recursive dir size", |b| {
        b.iter(|| {
            let _ = method.get_dir_size(path).unwrap();
        })
    });
}

fn bench_iterative(c: &mut Criterion) {
    let path = Path::new(".");
    let method = IterativeReader;

    c.bench_function("iterative dir size", |b| {
        b.iter(|| {
            let _ = method.get_dir_size(path).unwrap();
        })
    });
}

fn bench_parallel(c: &mut Criterion) {
    let path = Path::new(".");
    let method = ParallelReader;

    c.bench_function("parallel dir size", |b| {
        b.iter(|| {
            let _ = method.get_dir_size(path).unwrap();
        })
    });
}

criterion_group!(benches, bench_recursive, bench_iterative, bench_parallel);
criterion_main!(benches);
