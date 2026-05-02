use ark_ec::{CurveGroup, VariableBaseMSM};
use ark_secp256k1::{Affine, Fr, Projective};
use ark_std::UniformRand;

use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use std::time::Duration;

const SIZES: &[usize] = &[2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];

fn bench_msm(c: &mut Criterion) {
    let mut group = c.benchmark_group("ark_msm");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));
    group.sample_size(20);

    let mut rng = ark_std::test_rng();

    for &n in SIZES {
        let scalars: Vec<Fr> = (0..n).map(|_| Fr::rand(&mut rng)).collect();
        let bases_proj: Vec<Projective> = (0..n)
            .map(|_| Projective::rand(&mut rng))
            .collect();
        let bases: Vec<Affine> = Projective::normalize_batch(&bases_proj);

        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(BenchmarkId::new("multimult", n), &n, |b, _| {
            b.iter(|| Projective::msm(&bases, &scalars).unwrap())
        });

        group.bench_with_input(BenchmarkId::new("naive", n), &n, |b, &n| {
            b.iter(|| {
                let mut p = Projective::default();
                for i in 0..n {
                    p += bases[i] * scalars[i];
                }
                p
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_msm);
criterion_main!(benches);
