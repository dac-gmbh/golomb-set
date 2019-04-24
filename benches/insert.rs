#[macro_use]
extern crate criterion;

use {
    criterion::Criterion,
    golomb_set::GcsBuilder,
    rand::{rngs::SmallRng, FromEntropy, RngCore},
    sha1::Sha1,
};

fn insert(c: &mut Criterion) {
    let mut builder = GcsBuilder::<Sha1>::new(1000, 8);
    let mut rng = SmallRng::from_entropy();

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
