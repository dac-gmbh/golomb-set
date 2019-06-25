#[macro_use]
extern crate criterion;

use {
    criterion::Criterion,
    golomb_set::UnpackedGcs,
    rand_core::{RngCore, SeedableRng},
    rand_xorshift::XorShiftRng,
    twox_hash::XxHash,
};

fn contains_packed(c: &mut Criterion) {
    let gcs = {
        let mut unpacked = UnpackedGcs::<XxHash>::new(8000, 6);
        let mut rng = XorShiftRng::seed_from_u64(0);

        for _ in 0..8000 {
            let mut buf = [0u8; 128];
            rng.fill_bytes(&mut buf);
            unpacked.insert(&buf[..]).unwrap();
        }

        unpacked.pack()
    };

    c.bench_function("contains packed", move |b| {
        b.iter(|| gcs.contains(&[0, 1, 2, 3, 4, 5, 6, 7]))
    });
}

fn contains_unpacked(c: &mut Criterion) {
    let gcs = {
        let mut unpacked = UnpackedGcs::<XxHash>::new(8000, 6);
        let mut rng = XorShiftRng::seed_from_u64(0);

        for _ in 0..8000 {
            let mut buf = [0u8; 128];
            rng.fill_bytes(&mut buf);
            unpacked.insert(&buf[..]).unwrap();
        }

        unpacked
    };

    c.bench_function("contains unpacked", move |b| {
        b.iter(|| gcs.contains(&[0, 1, 2, 3, 4, 5, 6, 7]))
    });
}

criterion_group!(benches, contains_packed, contains_unpacked);
criterion_main!(benches);
