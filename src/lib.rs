//! A library for reading Blockland save files.
//! Generally tries to work around format errors like Blockland does.
//!
//! Create a [`Reader`](struct.Reader.html) from a
//! [`BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html) source to
//! read the save metadata and iterate over its bricks.
//!
//! ```rust
//! let file = BufReader::new(File::open("House.bls")?);
//! let reader = bl_save::Reader::new(file)?;
//!
//! for line in reader.description() {
//!     println!("{}", line);
//! }
//!
//! assert_eq!(reader.colors().len(), 64);
//! println!("Brick count: {}", reader.brick_count());
//!
//! for brick in reader {
//!     let brick = brick?;
//! }
//! ```

mod cp1252;
mod data;
mod escape;
mod read;

pub use data::{Brick, BrickBase};
pub use read::Reader;
