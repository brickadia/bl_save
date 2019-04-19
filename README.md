[![bl_save on crates.io][cratesio-image]][cratesio]
[![bl_save on docs.rs][docsrs-image]][docsrs]

[cratesio-image]: https://img.shields.io/crates/v/bl_save.svg
[cratesio]: https://crates.io/crates/bl_save
[docsrs-image]: https://docs.rs/bl_save/badge.svg
[docsrs]: https://docs.rs/bl_save

A library for reading Blockland save files.
Generally tries to work around format errors like Blockland does.

Create a [`Reader`](https://docs.rs/bl_save/*/bl_save/struct.Reader.html) from a
[`BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html) source to
read the save metadata and iterate over its bricks.

```rust
let file = BufReader::new(File::open("House.bls")?);
let reader = bl_save::Reader::new(file)?;

println!("Description: {}", reader.description());
println!("Brick count: {}", reader.brick_count());
assert_eq!(reader.colors().len(), 64);

for brick in reader {
    let brick = brick?;
}
```