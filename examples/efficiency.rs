use {
    golomb_set::UnpackedGcs,
    rand_core::{RngCore, SeedableRng},
    rand_xorshift::XorShiftRng,
    twox_hash::XxHash,
};

const NUM_ITEMS: usize = 1000;
const E: f64 = 2.718282;

fn main() {
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
        let mut gcs = UnpackedGcs::<XxHash>::new(NUM_ITEMS, 7);
        let mut prng = XorShiftRng::seed_from_u64(0);
        for _ in 0..NUM_ITEMS {
            let mut buf = [0u8; 32];
            prng.fill_bytes(&mut buf);
            gcs.insert(&buf).unwrap();
        }

        let mut gcs_buf = Vec::new();
        gcs.pack().write(&mut gcs_buf).unwrap();

        println!("GCS: {:?} bytes", gcs_buf.len());
    }

    // Theoretical minimum
    {
        println!(
            "Theoretical minimum: {:?} bytes",
            (1000.0 * 128f64.log2()) as u32 / 8
        )
    }
}
