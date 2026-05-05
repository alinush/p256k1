use p256k1::{point::Point, scalar::Scalar};

use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use rand_core::OsRng;
use std::time::Duration;

const SIZES: &[usize] = &[2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];

fn bench_msm(c: &mut Criterion) {
    let mut group = c.benchmark_group("msm");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));
    group.sample_size(20);

    for &n in SIZES {
        let mut rng = OsRng;
        let scalars: Vec<Scalar> = (0..n).map(|_| Scalar::random(&mut rng)).collect();
        let points: Vec<Point> = (0..n)
            .map(|_| Point::from(Scalar::random(&mut rng)))
            .collect();

        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(BenchmarkId::new("multimult", n), &n, |b, _| {
            b.iter(|| Point::multimult(scalars.clone(), points.clone()).unwrap())
        });

        group.bench_with_input(BenchmarkId::new("naive", n), &n, |b, &n| {
            b.iter(|| {
                let mut p = Point::identity();
                for i in 0..n {
                    p += scalars[i] * points[i];
                }
                p
            })
        });
    }

    group.finish();
}

/// Benchmarks the dedicated size-2 multiexp `na*P + ng*G` that ECDSA
/// verify / pubkey-recovery actually call inside libsecp256k1
/// (`secp256k1_ecmult`). It uses a precomputed odd-multiples table for `G`,
/// so it's faster than passing n=2 to `secp256k1_ecmult_multi_var`.
fn bench_ecmult_size2(c: &mut Criterion) {
    let mut rng = OsRng;
    let p = Point::from(Scalar::random(&mut rng));
    let na = Scalar::random(&mut rng);
    let ng = Scalar::random(&mut rng);

    let mut group = c.benchmark_group("ecmult_size2");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));
    group.sample_size(50);
    group.bench_function("na_P_plus_ng_G", |b| {
        b.iter(|| Point::ecmult(&p, &na, &ng))
    });
    group.finish();
}

criterion_group!(benches, bench_msm, bench_ecmult_size2);
criterion_main!(benches);
