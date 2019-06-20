#[macro_use]
extern crate criterion;

use {
    criterion::Criterion,
    golomb_set::GcsBuilder,
    rand_core::{RngCore, SeedableRng},
    rand_xorshift::XorShiftRng,
    twox_hash::XxHash,
};

fn insert(c: &mut Criterion) {
    let mut builder = GcsBuilder::<XxHash>::new(1000, 8);
    let mut rng = XorShiftRng::seed_from_u64(0);

    c.bench_function("insert", move |b| {
        b.iter(|| {
            let mut buf = [0u8; 128];
            rng.fill_bytes(&mut buf);
            builder.insert_unchecked(&buf);
        })
    });
}

criterion_group!(benches, insert);
criterion_main!(benches);
