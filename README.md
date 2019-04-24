# golomb-set

> A Golomb Coded Set implementation in Rust

Giovanni Bajo's blog post as well as their Python and C++ implementations were a huge help when writing this library, found [here](http://giovanni.bajo.it/post/47119962313/golomb-coded-sets-smaller-than-bloom-filters) and [here](https://github.com/rasky/gcs) respectively.

A GCS is a probabilistic data structure which is typically smaller than a bloom filter but slower to query. A sorted list of differences between samples of a random distribution will roughly have a geometric distribution, which makes for an optimal prefix for Golomb coding. Since a good hash will be randomly distributed, this encoding makes for efficient storage of hashed values.

## Example

```rust
use gcs::GcsBuilder;
use md5::Md5;

// Create a GCS where when 3 items are stored in the set, the
// probability of a false positive will be 1/2^5
let mut builder = GcsBuilder::new(3, 5);

// Insert the MD5 hashes of "alpha" and "bravo"
builder.insert::<Md5>(b"alpha")?;
builder.insert::<Md5>(b"bravo")?;

let gcs = builder.build();

assert!(gcs.contains(b"alpha"));
assert!(gcs.contains(b"bravo"));
assert!(!gcs.contains(b"charlie"));
```
