#[macro_use]
extern crate criterion;

use {
    criterion::Criterion,
    golomb_set::GcsBuilder,
    rand_core::{RngCore, SeedableRng},
    rand_xorshift::XorShiftRng,
    sha1::Sha1,
};

fn builder_fill(n: u64, p: u8) -> GcsBuilder<Sha1> {
    let mut builder = GcsBuilder::new(n, p);
    let mut rng = XorShiftRng::seed_from_u64(0);

    for _ in 0..n {
        let mut buf = [0u8; 128];
        rng.fill_bytes(&mut buf);
        builder.insert_unchecked(&buf);
    }

    builder
}

fn benchmark_1(c: &mut Criterion) {
    let builder = builder_fill(1, 6);

    c.bench_function("build 1", move |b| {
        // Cloning isn't ideal, need to figure out how to only measure .build()
        b.iter(|| builder.clone().build())
    });
}

fn benchmark_10(c: &mut Criterion) {
    let builder = builder_fill(10, 6);

    c.bench_function("build 10", move |b| {
        // Cloning isn't ideal, need to figure out how to only measure .build()
        b.iter(|| builder.clone().build())
    });
}

fn benchmark_100(c: &mut Criterion) {
    let builder = builder_fill(100, 6);

    c.bench_function("build 100", move |b| {
        // Cloning isn't ideal, need to figure out how to only measure .build()
        b.iter(|| builder.clone().build())
    });
}

criterion_group!(benches, benchmark_1, benchmark_10, benchmark_100);
criterion_main!(benches);
