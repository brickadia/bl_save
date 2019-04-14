use std::{
	env,
	fs::File,
	io::{self, BufReader},
};

fn main() -> io::Result<()> {
	let path = env::args().nth(1).expect("missing path");
	let reader = bl_save::Reader::new(BufReader::new(File::open(path)?))?;

	println!("Description:");
	for line in reader.description() {
		println!("{}", line);
	}

	let opaque_colors = reader.colors().iter().filter(|c| c.3 >= 1.0).count();
	println!("Opaque color count: {}", opaque_colors);

	println!("Expected brick count: {}", reader.brick_count());

	let mut read_bricks = 0;

	for brick in reader {
		let _brick = brick?;
		read_bricks += 1;
	}

	println!("Actual brick count: {}", read_bricks);

	Ok(())
}
