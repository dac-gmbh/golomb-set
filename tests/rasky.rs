// Checking that this library is compatible with this Python implementation: https://github.com/rasky/gcs

use {
    digest::{
        generic_array::{typenum::U4, GenericArray},
        Digest,
    },
    golomb_set::UnpackedGcs,
    md5::Md5,
    std::{
        fs::File,
        io::{BufRead, BufReader},
    },
};

pub struct Md5Trunc(Md5);

impl Digest for Md5Trunc {
    type OutputSize = U4;

    fn new() -> Self {
        Md5Trunc(Md5::new())
    }

    fn input<B: AsRef<[u8]>>(&mut self, data: B) {
        self.0.input(data);
    }

    fn chain<B: AsRef<[u8]>>(self, data: B) -> Self {
        Md5Trunc(self.0.chain(data))
    }

    fn result(self) -> GenericArray<u8, Self::OutputSize> {
        GenericArray::clone_from_slice(&self.0.result()[12..16])
    }

    fn result_reset(&mut self) -> GenericArray<u8, Self::OutputSize> {
        GenericArray::clone_from_slice(&self.0.result_reset()[12..16])
    }

    fn reset(&mut self) {
        self.0.reset();
    }

    fn output_size() -> usize {
        4
    }

    fn digest(data: &[u8]) -> GenericArray<u8, Self::OutputSize> {
        GenericArray::clone_from_slice(&Md5::digest(data)[12..16])
    }
}

#[test]
fn uuids_short_creation() {
    let mut gcs = UnpackedGcs::<Md5Trunc>::new(5, 10);

    let f = File::open("data/v4_uuids_short.txt").unwrap();
    let file = BufReader::new(&f);

    for line in file.lines() {
        let l = line.unwrap();
        gcs.insert(l.as_bytes()).unwrap();
    }

    let mut gcs_buf = Vec::new();
    gcs.pack().write(&mut gcs_buf).unwrap();

    assert_eq!(gcs_buf, include_bytes!("../data/v4_uuids_short.py.gcs"));
}

#[test]
fn uuids_1000_creation() {
    let mut gcs = UnpackedGcs::<Md5Trunc>::new(1000, 10);

    let f = File::open("data/v4_uuids.txt").unwrap();
    let file = BufReader::new(&f);

    for line in file.lines() {
        let l = line.unwrap();
        gcs.insert(l.as_bytes()).unwrap();
    }

    let mut gcs_buf = Vec::new();
    gcs.pack().write(&mut gcs_buf).unwrap();

    assert_eq!(gcs_buf, &include_bytes!("../data/v4_uuids.py.gcs")[..]);
}

#[test]
fn uuids_short_query_unpacked() {
    let gcs = {
        let mut unpacked = UnpackedGcs::<Md5Trunc>::new(5, 10);

        let f = File::open("data/v4_uuids_short.txt").unwrap();
        let file = BufReader::new(&f);

        for line in file.lines() {
            let l = line.unwrap();
            unpacked.insert(l.as_bytes()).unwrap();
        }

        unpacked
    };

    let f = File::open("data/v4_uuids_short.txt").unwrap();
    let file = BufReader::new(&f);

    for line in file.lines() {
        let l = line.unwrap();
        assert!(gcs.contains(l.as_bytes()))
    }
}

#[test]
fn uuids_1000_query_unpacked() {
    let gcs = {
        let mut unpacked = UnpackedGcs::<Md5Trunc>::new(1000, 10);

        let f = File::open("data/v4_uuids.txt").unwrap();
        let file = BufReader::new(&f);

        for line in file.lines() {
            let l = line.unwrap();
            unpacked.insert(l.as_bytes()).unwrap();
        }

        unpacked
    };

    let f = File::open("data/v4_uuids.txt").unwrap();
    let file = BufReader::new(&f);

    for line in file.lines() {
        let l = line.unwrap();
        assert!(gcs.contains(l.as_bytes()))
    }
}

#[test]
fn uuids_short_query_packed() {
    let gcs = {
        let mut unpacked = UnpackedGcs::<Md5Trunc>::new(5, 10);

        let f = File::open("data/v4_uuids_short.txt").unwrap();
        let file = BufReader::new(&f);

        for line in file.lines() {
            let l = line.unwrap();
            unpacked.insert(l.as_bytes()).unwrap();
        }

        unpacked.pack()
    };

    let f = File::open("data/v4_uuids_short.txt").unwrap();
    let file = BufReader::new(&f);

    for line in file.lines() {
        let l = line.unwrap();
        assert!(gcs.contains(l.as_bytes()).unwrap())
    }
}

#[test]
fn uuids_1000_query_packed() {
    let gcs = {
        let mut unpacked = UnpackedGcs::<Md5Trunc>::new(1000, 10);

        let f = File::open("data/v4_uuids.txt").unwrap();
        let file = BufReader::new(&f);

        for line in file.lines() {
            let l = line.unwrap();
            unpacked.insert(l.as_bytes()).unwrap();
        }

        unpacked.pack()
    };

    let f = File::open("data/v4_uuids.txt").unwrap();
    let file = BufReader::new(&f);

    for line in file.lines() {
        let l = line.unwrap();
        assert!(gcs.contains(l.as_bytes()).unwrap())
    }
}
