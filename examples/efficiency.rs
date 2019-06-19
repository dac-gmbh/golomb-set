use {
    golomb_set::GcsBuilder,
    md5::Md5,
    rand_core::{RngCore, SeedableRng},
    rand_xorshift::XorShiftRng,
};

const NUM_ITEMS: usize = 1000;
const E: f64 = 2.718282;

fn main() {
    let mut prng = XorShiftRng::seed_from_u64(0);

    let mut elements = Vec::<[u8; 32]>::with_capacity(NUM_ITEMS);
    for _ in 0..1000 {
        let mut buf = [0u8; 32];
        prng.fill_bytes(&mut buf);
        elements.push(buf);
    }

    println!("plain list: {:?} bytes", 1000 * 32);

    // Bloom filter
    {
        let p: f64 = 128.0;
        println!(
            "bloom filter: {:?} bytes",
            (NUM_ITEMS as f64 * E.log2() * p.log2()) as u32 / 8
        );
    }

    // GCS
    {
        let mut gcs = GcsBuilder::<Md5>::new(NUM_ITEMS as u64, 7);
        for elem in elements {
            gcs.insert_unchecked(&elem);
        }

        println!("GCS: {:?} bytes", gcs.build().as_bits().len() / 8);
    }

    // Theoretical minimum
    {
        println!(
            "Theoretical minimum: {:?} bytes",
            (1000.0 * 128f64.log2()) as u32 / 8
        )
    }
}
