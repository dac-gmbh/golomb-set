use {
    byteorder::{BigEndian, WriteBytesExt},
    golomb_set::UnpackedGcs,
    rand_core::{RngCore, SeedableRng},
    rand_xorshift::XorShiftRng,
    twox_hash::XxHash,
};

const TRIES: usize = 5000;
const ELEMENTS: u16 = 10000;
const PROBABILITY: u8 = 3;

fn main() {
    // GCS file generated with the following code:
    let gcs = {
        let mut gcs = UnpackedGcs::<XxHash>::new(ELEMENTS as usize, PROBABILITY);
        let mut buf = vec![0u8; 2];
        for element in 0..ELEMENTS {
            buf.write_u16::<BigEndian>(element).unwrap();
            gcs.insert(&buf).unwrap();
        }
        gcs.pack()
    };

    // Expected false probability of 12.5%
    let mut prng = XorShiftRng::seed_from_u64(0);
    let mut num = 0;
    let mut buf = [0u8; 4];
    for _ in 0..TRIES {
        prng.fill_bytes(&mut buf);
        if gcs.contains(&buf) {
            // None of the values we are trying were inserted, so any present
            // are false positives
            num += 1;
        }
    }

    println!(
        "Expected false probability rate: {:?}%",
        (1.0 / 2f64.powf(PROBABILITY as f64)) * 100.0
    );
    println!(
        "Actual rate: {:?}%",
        ((num as f64) / (TRIES as f64)) * 100f64
    )
}
