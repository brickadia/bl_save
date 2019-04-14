use crate::data::BrickBase;
use crate::Brick;
use std::io::{self, prelude::*};
use std::iter::Peekable;

const BRICK_COUNT_PREFIX: &str = "Linecount ";
const EXTRA_DATA_PREFIX: &str = "+-";

type Color = (f32, f32, f32, f32);
type Colors = [Color; 64];

pub struct Reader<R: BufRead> {
	brick_data: Peekable<BrickDataParser<Cp1252Lines<R>>>,
	description: Vec<String>,
	colors: Colors,
	brick_count: usize,
}

impl<R: BufRead> Reader<R> {
	pub fn new(r: R) -> io::Result<Self> {
		let mut lines = cp1252_lines(r);

		// This is a Blockland save file.
		// You probably shouldn't modify it cause you'll screw it up.
		lines.next().unwrap_or_else(|| Ok(String::from("")))?;

		// Description.
		let description_line_count = expect_line(&mut lines, "Missing description length")?
			.parse()
			.unwrap_or(0);
		if description_line_count > 1000 {
			return Err(invalid_data("Description is unreasonably long"));
		}
		let mut description = Vec::with_capacity(description_line_count);
		for _ in 0..description_line_count {
			description.push(expect_line(&mut lines, "Missing description line")?);
		}

		// Colors.
		let mut colors = [Default::default(); 64];
		for color in colors.iter_mut() {
			let line = lines.next().unwrap_or_else(|| Ok(String::from("")))?;
			// TODO: Parse colors
			let mut chars = line.chars();
			let r = float_from_chars(&mut chars);
			let g = float_from_chars(&mut chars);
			let b = float_from_chars(&mut chars);
			let a = float_from_chars(&mut chars);
			*color = (r, g, b, a);
		}

		// Brick count.
		let brick_count_line = lines.next().unwrap_or_else(|| Ok(String::from("")))?;
		if !brick_count_line.starts_with(BRICK_COUNT_PREFIX) {
			return Err(invalid_data("Invalid brick count line"));
		}
		let brick_count = brick_count_line[BRICK_COUNT_PREFIX.len()..]
			.parse()
			.unwrap_or(0);

		Ok(Self {
			brick_data: BrickDataParser(lines).peekable(),
			description,
			colors,
			brick_count,
		})
	}

	/// The description lines of the save file.
	/// This will likely become a single string in the future.
	pub fn description(&self) -> &[String] {
		&self.description
	}

	/// The colorset used by bricks in the save file.
	pub fn colors(&self) -> &Colors {
		&self.colors
	}

	/// The claimed brick count. Not guaranteed to be correct.
	pub fn brick_count(&self) -> usize {
		self.brick_count
	}
}

impl<R: BufRead> Iterator for Reader<R> {
	type Item = io::Result<Brick>;

	fn next(&mut self) -> Option<Self::Item> {
		let first = match self.brick_data.next() {
			Some(Ok(BrickLine::Base(data))) => data,
			Some(Ok(BrickLine::Extra(_))) => {
				panic!("previous iteration should have handled extra brick data")
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
				Some(Ok(BrickLine::Base(_))) | None => break,
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

		Some(Ok(brick))
	}
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
}

enum BrickExtra {
	Unknown(String),
}

fn invalid_data(error: &str) -> io::Error {
	io::Error::new(io::ErrorKind::InvalidData, error)
}

fn expect_line(
	lines: &mut impl Iterator<Item = io::Result<String>>,
	error: &str,
) -> io::Result<String> {
	lines.next().ok_or_else(|| invalid_data(error))?
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
	let mut word = String::new();

	while let Some(c) = iter.next() {
		if c == ' ' {
			break;
		}

		word.push(c);
	}

	word
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
