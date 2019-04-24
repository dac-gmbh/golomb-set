use {golomb_set::GcsBuilder, proptest::prelude::*, sha1::Sha1};

proptest! {
    #[test]
    fn add_query(bytes: Vec<u8>) {
        let gcs = {
            let mut builder = GcsBuilder::<Sha1>::new(10, 9);
            builder.insert_unchecked(&bytes);
            builder.build()
        };

        assert!(gcs.contains(&bytes));
    }

    #[test]
    fn invalid_query(a: Vec<u8>, b: Vec<u8>, n in 0u64..100000u64, p in 2u8..16) {
        if a == b {
            return Ok(());
        }
        let gcs = {
            let mut builder = GcsBuilder::<Sha1>::new(n, p);
            builder.insert_unchecked(&a);
            builder.build()
        };

        assert!(!gcs.contains(&b));
    }
}
