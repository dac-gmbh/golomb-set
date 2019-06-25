#[macro_use]
extern crate criterion;

use {
    criterion::Criterion,
    golomb_set::UnpackedGcs,
    rand_core::{RngCore, SeedableRng},
    rand_xorshift::XorShiftRng,
    twox_hash::XxHash,
};

fn unpacked_fill(n: usize, p: u8) -> UnpackedGcs<XxHash> {
    let mut unpacked = UnpackedGcs::new(n, p);
    let mut rng = XorShiftRng::seed_from_u64(0);

    for _ in 0..n {
        let mut buf = [0u8; 128];
        rng.fill_bytes(&mut buf);
        unpacked.insert(&buf[..]).unwrap();
    }

    unpacked
}

fn pack_1(c: &mut Criterion) {
    let unpacked = unpacked_fill(1, 6);

    c.bench_function("pack 1", move |b| b.iter(|| unpacked.pack()));
}

fn unpack_1(c: &mut Criterion) {
    let packed = unpacked_fill(1, 6).pack();

    c.bench_function("unpack 1", move |b| b.iter(|| packed.unpack()));
}

fn pack_100(c: &mut Criterion) {
    let unpacked = unpacked_fill(1, 6);

    c.bench_function("pack 100", move |b| b.iter(|| unpacked.pack()));
}

fn unpack_100(c: &mut Criterion) {
    let packed = unpacked_fill(1, 6).pack();

    c.bench_function("unpack 100", move |b| b.iter(|| packed.unpack()));
}

criterion_group!(benches, pack_1, unpack_1, pack_100, unpack_100);
criterion_main!(benches);
