#[macro_use]
extern crate doc_comment;

doctest!("../README.md");

use {golomb_set::UnpackedGcs, proptest::prelude::*, twox_hash::XxHash};

proptest! {
    #[test]
    fn add_query_unpacked(bytes: Vec<u8>) {
        let gcs = {
            let mut unpacked = UnpackedGcs::<XxHash>::new(10, 9);
            unpacked.insert(&bytes).unwrap();
            unpacked
        };

        assert!(gcs.contains(&bytes));
    }

    #[test]
    fn invalid_query_unpacked(a: Vec<u8>, b: Vec<u8>, n in 0i32..100000i32, p in 2u8..16) {
        if a == b {
            return Ok(());
        }
        let gcs = {
            let mut unpacked = UnpackedGcs::<XxHash>::new(n as usize, p);
            unpacked.insert(&a).unwrap();
            unpacked
        };

        assert!(!gcs.contains(&b));
    }
}
