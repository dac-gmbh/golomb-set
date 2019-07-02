#[macro_use]
extern crate doc_comment;

doctest!("../README.md");

use {golomb_set::UnpackedGcs, proptest::prelude::*, twox_hash::XxHash};

proptest! {
    #[test]
    fn add_query_unpacked_single(bytes: Vec<u8>) {
        let gcs = {
            let mut unpacked = UnpackedGcs::<XxHash>::new(10, 9);
            unpacked.insert(&bytes).unwrap();
            unpacked
        };

        assert!(gcs.contains(&bytes));
    }

    #[test]
    fn add_query_packed_single(bytes: Vec<u8>) {
        let gcs = {
            let mut unpacked = UnpackedGcs::<XxHash>::new(10, 9);
            unpacked.insert(&bytes).unwrap();
            unpacked.pack()
        };

        assert!(gcs.contains(&bytes).unwrap());
    }

    #[test]
    fn invalid_query_unpacked_single(a: Vec<u8>, b: Vec<u8>, n in 0i32..100000i32, p in 2u8..16) {
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

    #[test]
    fn invalid_query_packed_single(a: Vec<u8>, b: Vec<u8>, n in 0i32..10000i32, p in 2u8..16) {
        if a == b {
            return Ok(());
        }
        let gcs = {
            let mut unpacked = UnpackedGcs::<XxHash>::new(n as usize, p);
            unpacked.insert(&a).unwrap();
            unpacked.pack()
        };

        assert!(!gcs.contains(&b).unwrap());
    }

    // Tests the packing/unpacking roundtrip
    #[test]
    fn pack_roundtrip(n in 0usize..10000usize, p in 2u8..16, data: Vec<Vec<u8>>) {
        if n < data.len() {
            return Ok(());
        }

        let mut gcs = UnpackedGcs::<XxHash>::new(n, p);
        for elem in data {
            gcs.insert(elem).unwrap();
        }

        assert_eq!(gcs, gcs.pack().unpack().unwrap());
    }
}
