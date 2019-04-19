use crate::cp1252::BYTE_TO_CHAR;

pub fn collapse(dst: &mut String, chars: impl IntoIterator<Item = char>) {
    let mut chars = chars.into_iter();

    loop {
        match chars.next() {
            Some('\\') => collapse_one(dst, &mut chars),
            Some(c) => dst.push(c),
            None => break,
        }
    }
}

fn collapse_one(dst: &mut String, mut chars: impl Iterator<Item = char>) {
    match chars.next() {
        Some('x') => match chars.next() {
            Some(hex1) => match chars.next() {
                Some(hex2) => {
                    let mut src = String::with_capacity(2);
                    src.push(hex1);
                    src.push(hex2);
                    match u8::from_str_radix(&src, 16) {
                        Ok(byte) => dst.push(BYTE_TO_CHAR[usize::from(byte)]),
                        Err(_) => {
                            dst.push_str("\\x");
                            dst.push(hex1);
                            dst.push(hex2)
                        }
                    }
                }
                None => {
                    dst.push_str("\\x");
                    dst.push(hex1)
                }
            },
            None => dst.push_str("\\x"),
        },
        Some('c') => match chars.next() {
            Some('r') => dst.push(char::from(15)),
            Some('p') => dst.push(char::from(16)),
            Some('o') => dst.push(char::from(17)),
            Some('0') => {
                if dst.is_empty() {
                    dst.push(char::from(2));
                }
                dst.push(char::from(1))
            }
            Some('1') => dst.push(char::from(2)),
            Some('2') => dst.push(char::from(3)),
            Some('3') => dst.push(char::from(4)),
            Some('4') => dst.push(char::from(5)),
            Some('5') => dst.push(char::from(6)),
            Some('6') => dst.push(char::from(7)),
            Some('7') => dst.push(char::from(0xb)),
            Some('8') => dst.push(char::from(0xc)),
            Some('9') => dst.push(char::from(0xe)),
            Some(c) => {
                dst.push_str("\\c");
                dst.push(c)
            }
            None => dst.push_str("\\c"),
        },
        Some('r') => dst.push('\r'),
        Some('n') => dst.push('\n'),
        Some('t') => dst.push('\t'),
        Some(b) => dst.push(b),
        None => dst.push('\\'),
    }
}
