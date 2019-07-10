# golomb-set

> A Golomb Coded Set implementation in Rust

[![crates.io](https://img.shields.io/crates/v/golomb-set.svg)](https://crates.io/crates/golomb-set)
[![docs.rs](https://docs.rs/golomb-set/badge.svg)](https://docs.rs/golomb-set/)
[![Build Status](https://travis-ci.org/dac-gmbh/golomb-set.svg?branch=master)](https://travis-ci.org/dac-gmbh/golomb-set)
[![Dependabot Status](https://api.dependabot.com/badges/status?host=github&repo=dac-gmbh/golomb-set)](https://dependabot.com)

A Golomb Coded Set is a probabilistic data structure which is typically smaller than a bloom filter, at the cost of query performance.. A sorted list of differences between samples of a random distribution will roughly have a geometric distribution, which makes for an optimal prefix for Golomb-Rice coding. Since a good hash algorithm will be randomly distributed, this encoding makes for efficient storage of hashed values.

Giovanni Bajo's blog post as well as their Python and C++ implementations were a huge help when writing and testing this library, found [here](http://giovanni.bajo.it/post/47119962313/golomb-coded-sets-smaller-than-bloom-filters) and [here](https://github.com/rasky/gcs) respectively.

## Usage and Behaviour

There are 3 main parameters to select when creating a Golomb Coded Set: the hash algorithm, `N` and `P`. `N` is the desired maximum number of elements that will be inserted into the set, and `1 / 2 ^ P` is the desired probability of a false positive when the set is full. If fewer items have been inserted the real probability will be significantly lower.

The chosen hashing algorithm must have a uniform distribution (which is not the same as being cryptograpically secure) and the output length of the hash in bits must be greater than `log2(N * 2 ^ P)` bits. This is not currently enforced by the library and failing to do so could result in far more false positives than expected. Beyond meeting those requirements, selecting an algorithm for speed would be appropriate. If the hardware acceleration is present, CRC32 would be a good choice for up to a million elements and a false positive probability of 0.001%. For larger sets and/or lower probabilities a hashing algorithm with a longer output is needed.

## Example

```rust
use {golomb_set::UnpackedGcs, md5::Md5};

// Create a GCS where when 3 items are stored in the set, the
// probability of a false positive will be `1/(2^5)`, or 3.1%
let mut gcs = UnpackedGcs::<Md5>::new(3, 5);

// Insert the MD5 hashes of "alpha" and "bravo"
gcs.insert(b"alpha");
gcs.insert(b"bravo");

assert!(gcs.contains(b"alpha"));
assert!(gcs.contains(b"bravo"));
assert!(!gcs.contains(b"charlie"));

// Reduces memory footprint in exchange for slower querying
let gcs = gcs.pack();

assert!(gcs.contains(b"alpha"));
assert!(gcs.contains(b"bravo"));
assert!(!gcs.contains(b"charlie"));
```
