//! Giovanni Bajo's blog post as well as their Python and C++ implementations were a huge help
//! when writing this library, found
//! [here](http://giovanni.bajo.it/post/47119962313/golomb-coded-sets-smaller-than-bloom-filters)
//! and [here](https://github.com/rasky/gcs) respectively.
//!
//! A GCS is a probabilistic data structure which is typically smaller than a bloom filter but
//! slower to query. A sorted list of differences between samples of a random distribution will
//! roughly have a geometric distribution, which makes for an optimal prefix for Golomb coding.
//! Since a good hash will be randomly distributed, this encoding makes for efficient storage of
//! hashed values.
//!
//! ## Example
//!
//! ```rust
//! use golomb_set::UnpackedGcs;
//! use md5::Md5;
//!
//! // Create a GCS where when 3 items are stored in the set, the
//! // probability of a false positive will be 1/2^5
//! let mut gcs = UnpackedGcs::<Md5>::new(3, 5);
//!
//! // Insert the MD5 hashes of "alpha" and "bravo"
//! gcs.insert(b"alpha");
//! gcs.insert(b"bravo");
//!
//! assert!(gcs.contains(b"alpha"));
//! assert!(gcs.contains(b"bravo"));
//! assert!(!gcs.contains(b"charlie"));
//!
//! // Reduces memory footprint in exchange for slower querying
//! let gcs = gcs.pack();
//!
//! assert!(gcs.contains(b"alpha"));
//! assert!(gcs.contains(b"bravo"));
//! assert!(!gcs.contains(b"charlie"));
//! ```

#![deny(missing_docs)]

#[macro_use]
extern crate failure_derive;

use {
    bitvec::{
        prelude::{BigEndian, BitVec, LittleEndian},
        store::BitStore,
    },
    byteorder::ByteOrder,
    digest::Digest,
    failure::Fallible,
    num_integer::div_rem,
    std::{
        io::{self, Read, Write},
        marker::PhantomData,
    },
};

#[derive(Debug, Fail)]
enum GcsError {
    #[fail(display = "The limit for the number of elements has been reached")]
    LimitReached,
}

/// Builder for a GCS
#[derive(Clone, Debug, PartialEq)]
pub struct UnpackedGcs<D: Digest> {
    n: usize,
    p: u8,
    values: Vec<u64>,
    digest: PhantomData<D>,
}

impl<D: Digest> UnpackedGcs<D> {
    /// Creates a new UnpackedGcs from n and p, where 1/2^p is the probability
    /// of a false positive when n items have been inserted into the set
    pub fn new(n: usize, p: u8) -> Self {
        Self {
            n,
            p,
            values: Vec::new(),
            digest: PhantomData,
        }
    }

    /// Copies data from the reader and inserts into into the set.
    /// # Errors
    /// * If there is an error reading data from `reader`.
    /// * If more than `n` items have been inserted.
    pub fn insert_from_reader<R: Read>(&mut self, mut reader: R) -> Fallible<()> {
        let mut vec = Vec::new();
        reader.read_exact(&mut vec)?;
        self.insert(&vec)
    }

    /// Adds an entry to the set, and returns an error if more than N items are added
    pub fn insert<A: AsRef<[u8]>>(&mut self, input: A) -> Fallible<()> {
        if self.values.len() < self.n {
            self.values
                .push(digest_value::<D>(self.n as u64, self.p, input.as_ref()));
            self.values.sort();
            Ok(())
        } else {
            Err(GcsError::LimitReached.into())
        }
    }

    /// Returns whether or not an input is contained in the set. If false the
    /// input is definitely not present, if true the input is probably present
    pub fn contains<A: AsRef<[u8]>>(&self, input: A) -> bool {
        self.values
            .binary_search(&digest_value::<D>(self.n as u64, self.p, input.as_ref()))
            .is_ok()
    }

    /// Packs an `UnpackedGcs` into a `Gcs`
    ///
    /// This will will reduce the memory footprint, but reduce query
    /// performance
    pub fn pack(&self) -> Gcs<D> {
        let mut values = self.values.clone();

        // Sort then calculate differences
        values.sort();
        for i in (1..values.len()).rev() {
            values[i] -= values[i - 1];
        }

        // Apply golomb encoding
        let mut data = BitVec::<BigEndian, u8>::new();
        for val in values {
            data.append(&mut golomb_encode(val, self.p))
        }

        Gcs {
            n: self.n,
            p: self.p,
            data,
            digest: self.digest,
        }
    }
}

/// A Golomb-coded Set
#[derive(Clone, Debug, PartialEq)]
pub struct Gcs<D: Digest> {
    n: usize,
    p: u8,
    data: BitVec,
    digest: PhantomData<D>,
}

impl<D: Digest> Gcs<D> {
    /// Read a packed `Gcs` from any Reader
    pub fn from_reader<R: Read>(reader: &mut R, n: usize, p: u8) -> Fallible<Self> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        Ok(Self {
            n,
            p,
            data: BitVec::<BigEndian, u8>::from_vec(buf),
            digest: PhantomData,
        })
    }

    /// Writes a packed `Gcs` to a Writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_all(&self.data.clone().into_vec())
    }

    /// Returns whether or not an input is contained in the set. If false the
    /// input is definitely not present, if true the input is probably present
    pub fn contains<A: AsRef<[u8]>>(&self, _input: A) -> bool {
        false
    }

    /// Unpacks a `Gcs` into an `UnpackedGcs`
    ///
    /// This will will reduce the query performance, but improve query
    /// performance
    pub fn unpack(&self) -> UnpackedGcs<D> {
        let mut values = {
            let mut iter = self.data.iter().peekable();
            let mut values = Vec::new();

            while let Some(_) = iter.peek() {
                values.push(golomb_decode(&mut iter, self.p));
            }

            values
        };

        for i in 1..values.len() {
            values[i] += values[i - 1];
        }

        values.sort();

        UnpackedGcs {
            n: self.n,
            p: self.p,
            values,
            digest: self.digest,
        }
    }
}

/// Perform Golomb-Rice encoding of n, with modulus 2^p
///
/// # Panics
///
/// Panics if `p == 0`.
fn golomb_encode(n: u64, p: u8) -> BitVec {
    if p == 0 {
        panic!("p cannot be 0");
    }
    let (quo, rem) = div_rem(n, 2u64.pow(u32::from(p)));

    let mut out = BitVec::new();

    // Unary encoding of quotient
    for _ in 0..quo {
        out.push(true);
    }
    out.push(false);

    // Binary encoding of remainder in p bits
    // remove vec and change to big end?
    for i in (0..p).rev() {
        out.push(rem.get::<LittleEndian>(i.into()));
    }

    out
}

/// Perform Golomb-Rice decoding of n, with modulus 2^p
fn golomb_decode<I>(iter: &mut I, p: u8) -> u64
where
    I: Iterator<Item = bool>,
{
    // parse unary encoded quotient
    let mut quo = 0u64;
    while iter.next().unwrap() {
        quo += 1;
    }

    // parse binary encoded remainder
    let mut rem = 0u64;
    for _ in 0..p {
        if iter.next().unwrap() {
            rem += 1;
        }
        rem <<= 1;
    }
    rem >>= 1;

    // push quo * p + rem
    quo * 2u64.pow(u32::from(p)) + rem
}

fn digest_value<D: Digest>(n: u64, p: u8, input: &[u8]) -> u64 {
    let val = if D::output_size() < 8 {
        let mut buf = [0u8; 8];
        let digest = D::digest(input);
        for i in 0..D::output_size() {
            buf[i + D::output_size()] = digest[i];
        }

        byteorder::BigEndian::read_u64(&buf)
    } else {
        byteorder::BigEndian::read_u64(&D::digest(input)[..8])
    };

    val % (n * 2u64.pow(u32::from(p)))
}

#[cfg(test)]
mod tests {
    use {super::*, proptest::prelude::*, twox_hash::XxHash};

    proptest! {
        // Ranges need to be extended after improving performance
        #[test]
        fn golomb_single(n in 0u64..100000u64, p in 2u8..16) {
            assert_eq!(n, golomb_decode(&mut golomb_encode(n, p).iter().peekable(), p));
        }

        // Tests the packing/unpacking roundtrip
        #[test]
        fn pack_roundtrip(n in 0usize..100000usize, p in 2u8..16, data: Vec<Vec<u8>>) {
            if n >= data.len() {
                let mut gcs = UnpackedGcs::<XxHash>::new(n, p);
                for elem in data {
                    gcs.insert(elem).unwrap();
                }

                assert_eq!(gcs, gcs.pack().unpack());
            }
        }
    }
}
