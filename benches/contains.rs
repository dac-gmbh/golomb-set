#[macro_use]
extern crate criterion;

use {
    criterion::Criterion,
    golomb_set::GcsBuilder,
    rand_core::{RngCore, SeedableRng},
    rand_xorshift::XorShiftRng,
    sha1::Sha1,
};

fn contains(c: &mut Criterion) {
    let gcs = {
        let mut builder = GcsBuilder::<Sha1>::new(8000, 6);
        let mut rng = XorShiftRng::seed_from_u64(0);

        for _ in 0..8000 {
            let mut buf = [0u8; 128];
            rng.fill_bytes(&mut buf);
            builder.insert_unchecked(&buf);
        }

        builder.build()
    };

    c.bench_function("contains", move |b| {
        b.iter(|| gcs.contains(&[0, 1, 2, 3, 4, 5, 6, 7]))
    });
}

criterion_group!(benches, contains);
criterion_main!(benches);
