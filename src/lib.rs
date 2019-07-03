//! Golomb Coded Set is a probabilistic data structure which is typically smaller than a bloom
//! filter, at the cost of query performance.. A sorted list of differences between samples of a
//! random distribution will roughly have a geometric distribution, which makes for an optimal
//! prefix for Golomb-Rice coding. Since a good hash algorithm will be randomly distributed, this
//! encoding makes for efficient storage of hashed values.
//!
//! Giovanni Bajo's blog post as well as their Python and C++ implementations were a huge help when
//! writing and testing this library, found
//! [here](http://giovanni.bajo.it/post/47119962313golomb-coded-sets-smaller-than-bloom-filters)
//! and [here](https://github.com/rasky/gcs) respectively.
//!
//! ## Usage and Behaviour
//!
//! There are 3 main parameters to select when creating a Golomb Coded Set: the hash algorithm, `N`
//! and `P`. `N` is the desired maximum number of elements that will be inserted into the set, and
//! `1 / 2 ^ P` is the desired probability of a false positive when the set is full. If fewer items
//! have been inserted the real probability will be significantly lower.
//!
//! The chosen hashing algorithm must have a uniform distribution (which is not the same as being
//! cryptograpically secure) and the output length of the hash in bits must be greater than
//! `log2(N * 2 ^ P)` bits. This is not currently enforced by the library and failing to do so
//! could result in far more false positives than expected. Beyond meeting those requirements,
//! selecting an algorithm for speed would be appropriate. If the hardware acceleration is present,
//! CRC32 would be a good choice for up to a million elements and a false positive probability of
//! 0.001%. For larger sets and/or lower probabilities a hashing algorithm with a longer output is
//! needed.
//!
//! ## Example
//!
//! ```rust
//! use {golomb_set::UnpackedGcs, md5::Md5};
//!
//! // Create a GCS where when 3 items are stored in the set, the
//! // probability of a false positive will be `1/(2^5)`, or 3.1%
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
//! assert!(gcs.contains(b"alpha").unwrap());
//! assert!(gcs.contains(b"bravo").unwrap());
//! assert!(!gcs.contains(b"charlie").unwrap());
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

/// Errors that may occur when handling Golomb Coded Sets.
#[derive(Debug, Fail)]
pub enum GcsError {
    /// Returned when attempting to insert an additional element into an
    /// already full Golomb Coded Set.
    #[fail(display = "Limit for the number of elements has been reached")]
    LimitReached,
    /// The Golomb-Rice encoded sequence of bits could not be decoded, returned
    /// when unpacking or calling the `contains` method on a a packed GCS.
    #[fail(display = "Decoding failed due to invalid Golomb-Rice bit sequence")]
    DecodeError,
}

/// An unpacked Golomb Coded Set.
#[derive(Clone, Debug, PartialEq)]
pub struct UnpackedGcs<D: Digest> {
    n: usize,
    p: u8,
    values: Vec<u64>,
    digest: PhantomData<D>,
}

impl<D: Digest> UnpackedGcs<D> {
    /// Creates a new `UnpackedGcs` from `n` and `p`, where `1/2^p` is the probability
    /// of a false positive when n items have been inserted into the set.
    pub fn new(n: usize, p: u8) -> Self {
        Self {
            n,
            p,
            values: Vec::new(),
            digest: PhantomData,
        }
    }

    /// Copies data from the reader and inserts into into the set.
    ///
    /// # Errors
    /// * If there is an error reading data from `reader`.
    /// * If more than `n` items have been inserted.
    pub fn insert_from_reader<R: Read>(&mut self, mut reader: R) -> Fallible<()> {
        let mut vec = Vec::new();
        reader.read_exact(&mut vec)?;
        self.insert(&vec)
    }

    /// Adds an entry to the set, and returns an error if more than N items are added.
    ///
    /// # Errors
    /// * If more than `n` items have been inserted.
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
    /// input is definitely not present, if true the input is probably present.
    pub fn contains<A: AsRef<[u8]>>(&self, input: A) -> bool {
        self.values
            .binary_search(&digest_value::<D>(self.n as u64, self.p, input.as_ref()))
            .is_ok()
    }

    /// Packs an `UnpackedGcs` into a `Gcs`.
    ///
    /// This will will reduce the memory footprint, but also reduce query
    /// performance.
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

/// A packed Golomb-coded Set.
#[derive(Clone, Debug, PartialEq)]
pub struct Gcs<D: Digest> {
    n: usize,
    p: u8,
    data: BitVec,
    digest: PhantomData<D>,
}

impl<D: Digest> Gcs<D> {
    /// Read a packed `Gcs` from any Reader.
    ///
    /// # Errors
    /// * If there is an error reading data from `reader`.
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

    /// Writes a packed `Gcs` to a Writer.
    ///
    /// # Errors
    /// * If there is an error writing data to `writer`.
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_all(&self.data.clone().into_vec())
    }

    /// Returns whether or not an input is contained in the set. If false the
    /// input is definitely not present, if true the input is probably present.
    ///
    /// # Errors
    /// * If the inner data is not a valid Golomb-Rice encoding.
    pub fn contains<A: AsRef<[u8]>>(&self, input: A) -> Fallible<bool> {
        let input = digest_value::<D>(self.n as u64, self.p, input.as_ref());

        let mut iter = self.data.iter().peekable();

        let mut last = 0;

        while iter.peek().is_some() {
            let decoded = golomb_decode(&mut iter, self.p)?;

            if input == (decoded + last) {
                return Ok(true);
            } else {
                last += decoded;
            }
        }

        Ok(false)
    }

    /// Unpacks a `Gcs` into an `UnpackedGcs`.
    ///
    /// This will will increase query performance, but also increase the memory
    /// footprint.
    ///
    /// # Errors
    /// * If the inner data is not a valid Golomb-Rice encoding.
    pub fn unpack(&self) -> Fallible<UnpackedGcs<D>> {
        let mut values = {
            let mut iter = self.data.iter().peekable();
            let mut values = Vec::new();

            while iter.peek().is_some() {
                values.push(golomb_decode(&mut iter, self.p)?);
            }

            values
        };

        for i in 1..values.len() {
            values[i] += values[i - 1];
        }

        values.sort();

        Ok(UnpackedGcs {
            n: self.n,
            p: self.p,
            values,
            digest: self.digest,
        })
    }
}

/// Perform Golomb-Rice encoding of n, with modulus 2^p.
///
/// # Panics
/// * Panics if `p == 0`.
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

/// Perform Golomb-Rice decoding of n, with modulus 2^p.
///
/// # Errors
/// * If `iter` is not a valid Golomb-Rice encoding
fn golomb_decode<I>(iter: &mut I, p: u8) -> Fallible<u64>
where
    I: Iterator<Item = bool>,
{
    // parse unary encoded quotient
    let quo = iter.take_while(|i| *i).count() as u64;

    // parse binary encoded remainder
    let mut rem = 0u64;
    for _ in 0..p {
        match iter.next() {
            Some(true) => {
                rem += 1;
            }

            Some(false) => {}

            None => {
                return Err(GcsError::DecodeError.into());
            }
        }

        rem <<= 1;
    }
    rem >>= 1;

    // push quo * p + rem
    Ok(quo * 2u64.pow(u32::from(p)) + rem)
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
    use {super::*, proptest::prelude::*};

    proptest! {
        // Ranges need to be extended after improving performance
        #[test]
        fn golomb_single(n in 0u64..100000u64, p in 2u8..16) {
            assert_eq!(
                n,
                golomb_decode(&mut golomb_encode(n, p).iter().peekable(), p).unwrap()
            );
        }
    }
}
