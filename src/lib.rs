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
//! use golomb_set::GcsBuilder;
//! use md5::Md5;
//!
//! // Create a GCS where when 3 items are stored in the set, the
//! // probability of a false positive will be 1/2^5
//! let mut builder = GcsBuilder::<Md5>::new(3, 5);
//!
//! // Insert the MD5 hashes of "alpha" and "bravo"
//! builder.insert_unchecked(b"alpha");
//! builder.insert_unchecked(b"bravo");
//!
//! let gcs = builder.build();
//!
//! assert!(gcs.contains(b"alpha"));
//! assert!(gcs.contains(b"bravo"));
//! assert!(!gcs.contains(b"charlie"));
//! ```

#![deny(missing_docs)]

#[macro_use]
extern crate failure_derive;

use {
    bitvec::{BitVec, Bits},
    byteorder::ByteOrder,
    digest::Digest,
    num_integer::div_rem,
    std::{io::Read, marker::PhantomData},
};

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Fail)]
enum GcsError {
    #[fail(display = "The limit for the number of elements has been reached")]
    LimitReached,
}

/// Builder for a GCS
#[derive(Clone, Debug)]
pub struct GcsBuilder<D: Digest> {
    n: u64,
    p: u8,
    values: Vec<u64>,
    digest: PhantomData<D>,
}

impl<D: Digest> GcsBuilder<D> {
    /// Creates a new GcsBuilder from n and p, where n is the number of items
    /// to be stored in the set and 1/2^p is the probability of a false positive
    pub fn new(n: u64, p: u8) -> Self {
        GcsBuilder {
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
    /// ```no_run
    /// use std::{fs::{self, File}, io::Read};
    ///
    /// use golomb_set::GcsBuilder;
    /// use sha1::Sha1;
    ///
    /// let path = "src/lib.rs";
    ///
    /// let mut builder = GcsBuilder::<Sha1>::new(3, 5);
    /// builder.insert_from_reader(File::open(path)?);
    ///
    /// let gcs = builder.build();
    ///
    /// assert!(gcs.contains(&fs::read(path)?));
    ///
    ///# Ok::<_, std::io::Error>(())
    /// ```
    pub fn insert_from_reader<R: Read>(&mut self, mut reader: R) -> Result<()> {
        let mut vec = Vec::new();
        reader.read(&mut vec)?;
        self.insert(&vec)
    }

    /// Adds an entry to the set, and returns an error if more than N items are added
    pub fn insert<A: AsRef<[u8]>>(&mut self, input: A) -> Result<()> {
        if (self.values.len() as u64) < self.n {
            self.values
                .push(digest_value::<D>(self.n, self.p, input.as_ref()));
            Ok(())
        } else {
            Err(GcsError::LimitReached.into())
        }
    }

    /// Adds an entry to the set, does not error if more than N items are added
    pub fn insert_unchecked(&mut self, input: &[u8]) {
        self.values.push(digest_value::<D>(self.n, self.p, input));
    }

    /// Consumes the builder and creates the encoded set
    pub fn build(mut self) -> Gcs<D> {
        let mut out = BitVec::new();

        // Sort then calculate differences
        self.values.sort();
        for i in (1..self.values.len()).rev() {
            self.values[i] -= self.values[i - 1];
        }

        // Apply golomb encoding
        let mut bits = BitVec::<bitvec::BigEndian>::new();
        for val in self.values {
            bits.append(&mut golomb_encode(val, self.p))
        }
        out.append(&mut bits);

        Gcs::<D>::new(self.n, self.p, out)
    }
}

/// A Golomb-coded Set
pub struct Gcs<D: Digest> {
    n: u64,
    p: u8,
    bits: BitVec,
    digest: PhantomData<D>,
}

impl<D: Digest> Gcs<D> {
    /// Create a GCS from n, p and a BitVec of the Golomb-Rice encoded values,
    /// where n is the number of items the GCS was defined with and 1/2^p is
    /// the probability of a false positive
    pub fn new(n: u64, p: u8, bits: BitVec) -> Self {
        Gcs {
            n,
            p,
            bits,
            digest: PhantomData,
        }
    }

    /// Returns whether or not an input is contained in the set. If false the
    /// input is definitely not present, if true the input is probably present
    pub fn contains(&self, input: &[u8]) -> bool {
        let mut values = golomb_decode(self.bits.clone().iter().peekable(), self.p);

        for i in 1..values.len() {
            values[i] += values[i - 1];
        }

        values.contains(&digest_value::<D>(self.n, self.p, input))
    }

    /// Get the raw data bytes from a GCS
    pub fn as_bits(&self) -> &BitVec {
        &self.bits
    }

    /// Get the raw values encoded in the BitVec
    pub fn values(&self) -> Vec<u64> {
        golomb_decode(self.bits.clone().iter().peekable(), self.p)
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
        out.push(rem.get::<bitvec::LittleEndian>(i.into()));
    }

    out
}

/// Perform Golomb-Rice decoding of n, with modulus 2^p
fn golomb_decode<I>(iter: I, p: u8) -> Vec<u64>
where
    I: Iterator<Item = bool>,
{
    let mut out = Vec::<u64>::new();
    let mut iter = iter.peekable();

    while let Some(_) = iter.peek() {
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
        out.push(quo * 2u64.pow(u32::from(p)) + rem);
    }

    out
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
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Ranges need to be extended after improving performance
        #[test]
        fn golomb_single(n in 0u64..100000u64, p in 2u8..16) {
            assert_eq!(n, golomb_decode(golomb_encode(n, p).iter().peekable(), p)[0]);
        }
    }
}
