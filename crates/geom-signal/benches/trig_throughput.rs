use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geom_signal::{atan2_shafer, atan_shafer, sin_cos, sqrt, Scalar};

fn bench_trig(c: &mut Criterion) {
    let mut group = c.benchmark_group("trig_throughput");
    let val = black_box(Scalar::from_num(0.5));

    group.bench_function("sin_cos", |b| b.iter(|| sin_cos(val)));
    group.bench_function("sqrt", |b| b.iter(|| sqrt(val)));
    group.bench_function("atan_shafer", |b| b.iter(|| atan_shafer(val)));
    group.bench_function("atan2_shafer", |b| b.iter(|| atan2_shafer(val, val)));

    group.finish();
}

criterion_group!(benches, bench_trig);
criterion_main!(benches);
