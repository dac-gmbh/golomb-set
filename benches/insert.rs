#[macro_use]
extern crate criterion;

use {
    criterion::Criterion,
    golomb_set::UnpackedGcs,
    rand_core::{RngCore, SeedableRng},
    rand_xorshift::XorShiftRng,
    twox_hash::XxHash,
};

// Not really measuring much more than hash speed and Vec insertion speed
fn insert_unpacked(c: &mut Criterion) {
    let unpacked = UnpackedGcs::<XxHash>::new(10000, 8);
    let mut rng = XorShiftRng::seed_from_u64(0);

    c.bench_function("insert", move |b| {
        b.iter(|| {
            let mut buf = [0u8; 128];
            rng.fill_bytes(&mut buf);
            unpacked.clone().insert(&buf[..]).unwrap();
        })
    });
}

// Measures same as packed, plus round trip packing speed, still not a useful metric
fn insert_packed(c: &mut Criterion) {
    let packed = UnpackedGcs::<XxHash>::new(10000, 8).pack();
    let mut rng = XorShiftRng::seed_from_u64(0);

    c.bench_function("insert", move |b| {
        b.iter(|| {
            let mut buf = [0u8; 128];
            rng.fill_bytes(&mut buf);
            let mut unpacked = packed.clone().unpack();
            unpacked.insert(&buf[..]).unwrap();
            let _packed = unpacked.pack();
        })
    });
}

criterion_group!(benches, insert_unpacked, insert_packed);
criterion_main!(benches);
