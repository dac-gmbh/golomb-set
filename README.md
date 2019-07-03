# golomb-set

> A Golomb Coded Set implementation in Rust

[![crates.io](https://img.shields.io/crates/v/golomb-set.svg)](https://crates.io/crates/golomb-set)
[![docs.rs](https://docs.rs/golomb-set/badge.svg)](https://docs.rs/golomb-set/)
[![Build Status](https://travis-ci.org/dac-gmbh/golomb-set.svg?branch=master)](https://travis-ci.org/dac-gmbh/golomb-set)
[![Dependabot Status](https://api.dependabot.com/badges/status?host=github&repo=dac-gmbh/golomb-set)](https://dependabot.com)

Giovanni Bajo's blog post as well as their Python and C++ implementations were a huge help when writing this library, found [here](http://giovanni.bajo.it/post/47119962313/golomb-coded-sets-smaller-than-bloom-filters) and [here](https://github.com/rasky/gcs) respectively.

A GCS is a probabilistic data structure which is typically smaller than a bloom filter but slower to query. A sorted list of differences between samples of a random distribution will roughly have a geometric distribution, which makes for an optimal prefix for Golomb coding. Since a good hash will be randomly distributed, this encoding makes for efficient storage of hashed values.

## Example

```rust
use golomb_set::UnpackedGcs;
use md5::Md5;

// Create a GCS where when 3 items are stored in the set, the
// probability of a false positive will be 1/2^5
let mut gcs = UnpackedGcs::<Md5>::new(3, 5);

// Insert the MD5 hashes of "alpha" and "bravo"
gcs.insert(b"alpha");
gcs.insert(b"bravo");

assert!(gcs.contains(b"alpha"));
assert!(gcs.contains(b"bravo"));
assert!(!gcs.contains(b"charlie"));

// Reduces memory footprint in exchange for slower querying
let gcs = gcs.pack();

assert!(gcs.contains(b"alpha").unwrap());
assert!(gcs.contains(b"bravo").unwrap());
assert!(!gcs.contains(b"charlie").unwrap());
```
