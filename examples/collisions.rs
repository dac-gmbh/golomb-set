use {
    golomb_set::GcsBuilder,
    rand_core::{RngCore, SeedableRng},
    rand_xorshift::XorShiftRng,
    twox_hash::XxHash,
};

const COUNT: usize = 10000;

fn main() {
    let mut gcs = GcsBuilder::<XxHash>::new(std::u16::MAX.into(), 3);
    for a in 0..255 {
        for b in 0..255 {
            gcs.insert_unchecked(&[a, b]);
        }
    }
    let gcs = gcs.build();

    // Expected false probability of 12.5%
    let mut prng = XorShiftRng::seed_from_u64(0);
    let mut num = 0;
    let mut buf = [0u8, 8];
    for _ in 0..COUNT {
        prng.fill_bytes(&mut buf);
        if gcs.contains(&buf) {
            num += 1;
        }
    }

    println!("Expected false probability rate: 12.5%");
    println!("Actual rate: {:?}%", (num / COUNT) * 100)
}
