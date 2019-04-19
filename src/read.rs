use crate::{data::BrickBase, escape::collapse, Brick};
use std::io::{self, prelude::*};
use std::iter::Peekable;

const LINECOUNT_PREFIX: &str = "Linecount ";
const EXTRA_DATA_PREFIX: &str = "+-";

type Color = (f32, f32, f32, f32);
type Colors = [Color; 64];

/// Reads save files.
///
/// Metadata including the description, colors and usually the brick count
/// is available on construction. Iterating over the reader yields the bricks.
///
pub struct Reader<R: BufRead> {
	brick_data: Peekable<BrickDataParser<Cp1252Lines<R>>>,
	description: String,
	colors: Colors,
	brick_count: Option<usize>,
}

impl<R: BufRead> Reader<R> {
	/// Construct a new instance from a
	/// [`BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html) source
	/// and immediately read metadata.
	///
	/// ```rust
	/// let file = BufReader::new(File::open("House.bls")?);
	/// let reader = bl_save::Reader::new(file)?;
	/// ```
	pub fn new(r: R) -> io::Result<Self> {
		let mut lines = cp1252_lines(r);

		// This is a Blockland save file.
		// You probably shouldn't modify it cause you'll screw it up.
		read_line(&mut lines)?;

		// Description.
		let description_line_count = read_line(&mut lines)?.parse().unwrap_or(0);
		if description_line_count > 1000 {
			return Err(invalid_data("Description is unreasonably long"));
		}
		let mut description_escaped = String::new();
		for line_index in 0..description_line_count {
			if line_index > 0 {
				description_escaped.push('\n');
			}
			description_escaped.push_str(&read_line(&mut lines)?);
		}
		let mut description = String::new();
		collapse(&mut description, description_escaped.chars());

		// Colors.
		let mut colors = [Default::default(); 64];
		for color in colors.iter_mut() {
			let line = read_line(&mut lines)?;
			let mut chars = line.chars();
			let r = float_from_chars(&mut chars);
			let g = float_from_chars(&mut chars);
			let b = float_from_chars(&mut chars);
			let a = float_from_chars(&mut chars);
			*color = (r, g, b, a);
		}

		let mut brick_data = BrickDataParser(lines).peekable();

		// Get the brick count early, if possible. It's usually the first line.
		let mut brick_count = None;

		if let Some(Ok(BrickLine::Linecount(_))) | Some(Err(_)) = brick_data.peek() {
			match brick_data.next() {
				Some(Ok(BrickLine::Linecount(count))) => brick_count = Some(count),
				Some(Err(e)) => return Err(e),
				_ => unreachable!(),
			}
		}

		Ok(Self {
			brick_data,
			description,
			colors,
			brick_count,
		})
	}

	/// The description of the save file.
	/// The reader will refuse to read more than 1,000 lines.
	pub fn description(&self) -> &str {
		&self.description
	}

	/// The colorset used by bricks in the save file.
	pub fn colors(&self) -> &Colors {
		&self.colors
	}

	/// The claimed brick count, if available. Not guaranteed to be correct.
	///
	/// The reader will attempt to make this available on construction,
	/// but that isn't guaranteed if the brick count is in a non-standard location or absent.
	/// It may become available during brick iteration.
	pub fn brick_count(&self) -> Option<usize> {
		self.brick_count
	}
}

impl<R: BufRead> Iterator for Reader<R> {
	type Item = io::Result<Brick>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let first = match self.brick_data.next() {
				Some(Ok(BrickLine::Base(data))) => data,
				Some(Ok(BrickLine::Extra(_))) => {
					panic!("previous iteration should have handled extra brick data")
				}
				Some(Ok(BrickLine::Linecount(count))) => {
					self.brick_count = Some(count);
					continue;
				}
				Some(Err(e)) => return Some(Err(e)),
				None => return None,
			};

			let mut brick = Brick {
				base: first,
				unknown_extra: Vec::new(),
			};

			loop {
				match self.brick_data.peek() {
					Some(Ok(BrickLine::Extra(_))) => {}
					Some(Ok(_)) | None => break,
					Some(Err(_)) => {
						let e = match self.brick_data.next() {
							Some(Err(e)) => e,
							_ => panic!("variant changed from peek() to next()"),
						};
						return Some(Err(e));
					}
				}

				let extra = match self.brick_data.next() {
					Some(Ok(BrickLine::Extra(extra))) => extra,
					_ => panic!("variant changed from peek() to next()"),
				};

				match extra {
					BrickExtra::Unknown(s) => brick.unknown_extra.push(s),
				}
			}

			return Some(Ok(brick));
		}
	}
}

fn read_line(mut lines: impl Iterator<Item = io::Result<String>>) -> io::Result<String> {
	lines.next().unwrap_or_else(|| Ok(String::from("")))
}

struct BrickDataParser<L>(L);

impl<L: Iterator<Item = io::Result<String>>> Iterator for BrickDataParser<L> {
	type Item = io::Result<BrickLine>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|r| r.and_then(parse_brick_data_line))
	}
}

fn parse_brick_data_line(line: String) -> io::Result<BrickLine> {
	if line.starts_with(EXTRA_DATA_PREFIX) {
		Ok(BrickLine::Extra(BrickExtra::Unknown(line)))
	} else if line.starts_with(LINECOUNT_PREFIX) {
		let brick_count = line[LINECOUNT_PREFIX.len()..].parse().unwrap_or(0);
		Ok(BrickLine::Linecount(brick_count))
	} else {
		let quote_index = line
			.find('"')
			.ok_or_else(|| invalid_data("Invalid brick line"))?;
		let ui_name = String::from(&line[..quote_index]);

		let mut chars = line[quote_index + '"'.len_utf8()..].chars();
		expect_eq_next(&mut chars, ' ', "Invalid brick line")?;

		// TODO: Handle invalid values for angle, color_index,
		// color_fx and shape_fx

		let x = float_from_chars(&mut chars);
		let y = float_from_chars(&mut chars);
		let z = float_from_chars(&mut chars);
		let angle = int_from_chars(&mut chars) as u8;
		let is_baseplate = bool_from_chars(&mut chars);
		let color_index = int_from_chars(&mut chars) as u8;
		let print = take_word_consume_space(&mut chars);
		let color_fx = int_from_chars(&mut chars) as u8;
		let shape_fx = int_from_chars(&mut chars) as u8;
		let raycasting = bool_from_chars(&mut chars);
		let collision = bool_from_chars(&mut chars);
		let rendering = bool_from_chars(&mut chars);

		Ok(BrickLine::Base(BrickBase {
			ui_name,
			position: (x, y, z),
			angle,
			is_baseplate,
			color_index,
			print,
			color_fx,
			shape_fx,
			raycasting,
			collision,
			rendering,
		}))
	}
}

enum BrickLine {
	Base(BrickBase),
	Extra(BrickExtra),
	Linecount(usize),
}

enum BrickExtra {
	Unknown(String),
}

fn invalid_data(error: &str) -> io::Error {
	io::Error::new(io::ErrorKind::InvalidData, error)
}

fn expect_next<T>(iter: &mut impl Iterator<Item = T>, error: &str) -> io::Result<T> {
	iter.next().ok_or_else(|| invalid_data(error))
}

fn expect_eq_next<T: PartialEq>(
	iter: &mut impl Iterator<Item = T>,
	cmp: T,
	error: &str,
) -> io::Result<()> {
	if expect_next(iter, error)? != cmp {
		return Err(invalid_data(error));
	}
	Ok(())
}

fn take_word_consume_space(iter: &mut impl Iterator<Item = char>) -> String {
	iter.take_while(|c| *c != ' ').collect()
}

fn float_from_chars(chars: &mut impl Iterator<Item = char>) -> f32 {
	take_word_consume_space(chars).parse().unwrap_or(0.0)
}

fn int_from_chars(chars: &mut impl Iterator<Item = char>) -> i32 {
	take_word_consume_space(chars).parse().unwrap_or(0)
}

fn bool_from_chars(chars: &mut impl Iterator<Item = char>) -> bool {
	int_from_chars(chars) != 0
}

fn cp1252_lines<R: BufRead>(r: R) -> Cp1252Lines<R> {
	Cp1252Lines(r)
}

struct Cp1252Lines<R>(R);

impl<R: BufRead> Iterator for Cp1252Lines<R> {
	type Item = io::Result<String>;

	fn next(&mut self) -> Option<Self::Item> {
		let mut buf = Vec::new();
		match self.0.read_until(b'\n', &mut buf) {
			Ok(0) => None,
			Ok(_n) => {
				if buf.iter().last() == Some(&b'\n') {
					buf.pop();
					if buf.iter().last() == Some(&b'\r') {
						buf.pop();
					}
				}
				Some(Ok(buf
					.into_iter()
					.map(|b| crate::cp1252::BYTE_TO_CHAR[b as usize])
					.collect()))
			}
			Err(e) => Some(Err(e)),
		}
	}
}
