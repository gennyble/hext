mod error;

pub use crate::error::Error;
use bitvec::prelude::*;
use error::InvalidHeaderKind;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq)]
struct Header {
    bitorder: BitOrder,
    byteorder: ByteOrder,
    pad_bits: bool,
}

#[derive(Debug, PartialEq)]
enum BitOrder {
    Msb0,
    Lsb0,
}

impl BitOrder {
    fn from_str<S: AsRef<str>>(s: S) -> Option<Self> {
        if s.as_ref() == "msb0" {
            Some(BitOrder::Msb0)
        } else if s.as_ref() == "lsb0" {
            Some(BitOrder::Lsb0)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq)]
enum ByteOrder {
    LittleEndian,
    BigEndian,
}

impl ByteOrder {
    fn from_str<S: AsRef<str>>(s: S) -> Option<Self> {
        if s.as_ref() == "big-endian" {
            Some(ByteOrder::BigEndian)
        } else if s.as_ref() == "little-endian" {
            Some(ByteOrder::LittleEndian)
        } else {
            None
        }
    }
}

pub struct Hext {
    parsed: Vec<u8>,
}

impl Hext {
    pub fn new() -> Self {
        Self { parsed: vec![] }
    }

    pub fn parse<S: AsRef<str>>(mut self, raw: S) -> Result<Vec<u8>, Error> {
        let mut chars = raw.as_ref().chars().peekable();

        // Clear through any leading comments or blank lines
        Self::skip_nondata(&mut chars);

        let header: Header;
        loop {
            match chars.next() {
                Some('~') => {
                    header = Self::parse_header(Self::consume_line(&mut chars))?;
                    break;
                }
                Some(_) => return Err(Error::NoHeader),
                None => return Ok(self.parsed), //todo: is this an error?
            }
        }

        let mut bits: BitVec<Msb0, u8> = BitVec::new();
        let mut state = State::ReadingHex;

        loop {
            match state {
                State::ReadingHex => match chars.next_if(|&c| c != '.') {
                    Some('#') => Self::skip_line(&mut chars),
                    Some(c) if c.is_whitespace() => continue,

                    Some(high) if high.is_ascii_hexdigit() => {
                        match chars.next_if(|&c| c.is_ascii_hexdigit()) {
                            Some(low) => self.parsed.push(
                                ((high.to_digit(16).unwrap() * 16) + low.to_digit(16).unwrap())
                                    as u8,
                            ),
                            None => return Err(Error::IncompleteOctet),
                        }
                    }
                    Some('\'') => state = State::ReadingLiteral,
                    Some(c) => return Err(Error::InvalidCharacter(c)),

                    None => match chars.peek() {
                        Some('.') => state = State::ReadingBinary,
                        Some(_) => unreachable!(),
                        None => return Ok(self.parsed),
                    },
                },

                State::ReadingLiteral => match chars.next() {
                    Some('\'') => state = State::ReadingHex,
                    Some('\\') => match chars.next() {
                        Some(c) => match Self::escape(c) {
                            Some(c) => self.parsed.push(c as u8),
                            None => return Err(Error::InvalidEscape(c)),
                        },
                        None => return Err(Error::UnclosedStringLiteral),
                    },
                    Some('\n') => return Err(Error::UnclosedStringLiteral),
                    Some(c) => {
                        let mut encode = vec![0; c.len_utf8()];
                        c.encode_utf8(&mut encode);
                        self.parsed.extend_from_slice(&encode)
                    }
                    None => return Err(Error::UnclosedStringLiteral),
                },

                State::ReadingBinary => match chars.next_if(|&c| c == '.') {
                    Some('.') => loop {
                        match chars
                            .next_if(|&c| c == '1' || c == '0' || c == '#' || c.is_whitespace())
                        {
                            Some('0') => bits.push(false),
                            Some('1') => bits.push(true),
                            Some('#') => Self::skip_line(&mut chars),
                            Some(c) if c.is_whitespace() => {
                                Self::skip_nondata(&mut chars);
                                break;
                            }
                            Some(_) => return Err(Error::GarbageCharacterInBitstream),
                            None => break,
                        }
                    },
                    Some(_) => unreachable!(),
                    None => {
                        if bits.len() % 8 != 0 {
                            if !header.pad_bits {
                                eprintln!("{}", bits.len());
                                return Err(Error::UnalignedBits);
                            } else {
                                while bits.len() % 8 != 0 {
                                    bits.insert(0, false);
                                }
                            }
                        }

                        self.parsed.extend_from_slice(bits.as_raw_slice());
                        bits = BitVec::new();

                        state = State::ReadingHex;
                    }
                },
            }
        }
    }

    fn parse_header<S: AsRef<str>>(string: S) -> Result<Header, Error> {
        let splits: Vec<&str> = string.as_ref().trim_end().split(' ').collect();

        let mut bitorder = None;
        let mut byteorder = None;
        let mut pad_bits = false;

        for split in splits {
            match split {
                "msb0" => {
                    if bitorder.replace(BitOrder::Msb0).is_some() {
                        return Err(InvalidHeaderKind::TwoBitOrder.into());
                    }
                }
                "lsb0" => {
                    if bitorder.replace(BitOrder::Lsb0).is_some() {
                        return Err(InvalidHeaderKind::TwoBitOrder.into());
                    }
                }
                "big-endian" => {
                    if byteorder.replace(ByteOrder::BigEndian).is_some() {
                        return Err(InvalidHeaderKind::TwoByteOrder.into());
                    }
                }
                "little-endian" => {
                    if byteorder.replace(ByteOrder::LittleEndian).is_some() {
                        return Err(InvalidHeaderKind::TwoByteOrder.into());
                    }
                }
                "padbits" => pad_bits = true,
                _ => return Err(InvalidHeaderKind::InvalidProperty(split.into()).into()),
            }
        }

        if bitorder.is_none() {
            return Err(InvalidHeaderKind::NoBitOrder.into());
        } else if byteorder.is_none() {
            return Err(InvalidHeaderKind::NoByteOrder.into());
        } else {
            Ok(Header {
                bitorder: bitorder.unwrap(),
                byteorder: byteorder.unwrap(),
                pad_bits,
            })
        }
    }

    fn escape(c: char) -> Option<char> {
        match c {
            '\'' => Some('\''),
            '\\' => Some('\\'),
            'n' => Some('\n'),
            'r' => Some('\r'),
            't' => Some('\t'),
            _ => None,
        }
    }

    fn skip_nondata(mut chars: &mut Peekable<Chars>) {
        loop {
            match chars.peek() {
                Some('#') => Self::skip_line(&mut chars),
                Some(c) if c.is_whitespace() => {
                    chars.next();
                }
                _ => return,
            };
        }
    }

    fn skip_line(chars: &mut Peekable<Chars>) {
        chars.find(|&c| c == '\n');
    }

    fn consume_line(chars: &mut Peekable<Chars>) -> String {
        chars.take_while(|&c| c != '\n').collect()
    }
}

enum State {
    ReadingHex,
    ReadingBinary,
    ReadingLiteral,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pares_header_success() {
        // Recognizes the keytwords...
        assert_eq!(
            Hext::parse_header("msb0 big-endian").unwrap(),
            Header {
                byteorder: ByteOrder::BigEndian,
                bitorder: crate::BitOrder::Msb0,
                pad_bits: false
            }
        );

        assert_eq!(
            Hext::parse_header("lsb0 little-endian").unwrap(),
            Header {
                byteorder: ByteOrder::LittleEndian,
                bitorder: crate::BitOrder::Lsb0,
                pad_bits: false
            }
        );

        // ...In either order
        assert_eq!(
            Hext::parse_header("big-endian lsb0").unwrap(),
            Header {
                byteorder: ByteOrder::BigEndian,
                bitorder: crate::BitOrder::Lsb0,
                pad_bits: false
            }
        );
    }

    #[test]
    fn parse_header_fail_twobits() {
        assert_eq!(
            Hext::parse_header("lsb0 msb0"),
            Err(InvalidHeaderKind::TwoBitOrder.into())
        )
    }

    #[test]
    fn parse_header_fail_twobytes() {
        assert_eq!(
            Hext::parse_header("little-endian big-endian"),
            Err(InvalidHeaderKind::TwoByteOrder.into())
        )
    }

    #[test]
    fn parse_header_fail_nobits() {
        assert_eq!(
            Hext::parse_header("big-endian"),
            Err(InvalidHeaderKind::NoBitOrder.into())
        )
    }

    #[test]
    fn parse_header_fail_nobytes() {
        assert_eq!(
            Hext::parse_header("msb0"),
            Err(InvalidHeaderKind::NoByteOrder.into())
        )
    }

    #[test]
    fn parse_header_fail_invalidproperty() {
        assert_eq!(
            Hext::parse_header("lsb0 big-endian invalidproperty"),
            Err(InvalidHeaderKind::InvalidProperty("invalidproperty".into()).into())
        )
    }

    //## Bytes tests ##
    #[test]
    fn test_onebyte() {
        let test = "~little-endian msb0\n41";
        let cmp = vec![0x41];

        assert_eq!(Hext::new().parse(&test).unwrap(), cmp);
    }

    #[test]
    fn test_only_comment() {
        let test = "~little-endian msb0\n# Comment";

        assert_eq!(Hext::new().parse(&test).unwrap(), vec![]);
    }

    #[test]
    fn test_1byte_comment() {
        let test = "~little-endian msb0\n41 #A";
        let cmp = vec![0x41];

        assert_eq!(Hext::new().parse(&test).unwrap(), cmp);
    }

    #[test]
    fn test_byte_nospace_comment() {
        let test = "~little-endian msb0\n41#A";
        let cmp = vec![0x41];

        assert_eq!(Hext::new().parse(&test).unwrap(), cmp);
    }

    #[test]
    fn test_2byte_multiline() {
        let test = "~little-endian msb0\n41\n42";
        let cmp = vec![0x41, 0x42];

        assert_eq!(Hext::new().parse(&test).unwrap(), cmp);
    }

    #[test]
    fn test_2bytes_nospace() {
        let test = "~little-endian msb0\n4142";
        let cmp = vec![0x41, 0x42];

        assert_eq!(Hext::new().parse(&test).unwrap(), cmp);
    }

    //## Bit Tests ##
    #[test]
    fn test_8bits() {
        let test = "~little-endian msb0\n.01000001";
        let cmp = vec![0x41];

        assert_eq!(Hext::new().parse(&test).unwrap(), cmp);
    }

    #[test]
    fn test_8bits_hex10() {
        let test = "~little-endian msb0\n.01000001 10";
        let cmp = vec![0x41, 0x10];

        assert_eq!(Hext::new().parse(&test).unwrap(), cmp);
    }

    #[test]
    fn test_8bit_comment() {
        let test = "~little-endian msb0\n.01000001 # A";
        let cmp = vec![0x41];

        assert_eq!(Hext::new().parse(&test).unwrap(), cmp)
    }

    #[test]
    fn test_8bit_nospace_comment() {
        let test = "~little-endian msb0\n.01000001#A";
        let cmp = vec![0x41];

        assert_eq!(Hext::new().parse(&test).unwrap(), cmp);
    }

    #[test]
    fn test_1bit() {
        let test = "~little-endian msb0 padbits\n.1";
        let cmp = vec![0x01];

        assert_eq!(Hext::new().parse(&test).unwrap(), cmp);
    }

    #[test]
    fn test_8bits_halved_space() {
        let test_space = "~little-endian msb0\n.0100 .0010";
        let cmp = vec![0x42];

        assert_eq!(Hext::new().parse(&test_space).unwrap(), cmp);
    }

    #[test]
    fn test_8bits_halved_line() {
        let test_line = "~little-endian msb0\n.0100\n.0010";
        let cmp = vec![0x42];

        assert_eq!(Hext::new().parse(&test_line).unwrap(), cmp);
    }

    #[test]
    fn test_8bits_halved_line_comments() {
        let test_line_comments = "~little-endian msb0\n.0100#Half of capital letter\n.0010 # B";
        let cmp = vec![0x42];

        assert_eq!(Hext::new().parse(&test_line_comments).unwrap(), cmp);
    }

    #[test]
    fn test_1bit_then_byte() {
        let test = "~little-endian msb0 padbits\n.1 41";
        let cmp = vec![0x01, 0x41];

        assert_eq!(Hext::new().parse(&test).unwrap(), cmp);
    }

    //## Literal Tests ##
    #[test]
    fn literal_multibyte() {
        let test = "~big-endian lsb0\n'🥺'";
        let cmp = vec![0xf0, 0x9f, 0xa5, 0xba];

        assert_eq!(Hext::new().parse(&test).unwrap(), cmp);
    }

    //## Everything ##
    #[test]
    fn everything() {
        let to_parse = std::fs::read_to_string("tests/everything.hxt").unwrap();
        let cmp = std::fs::read_to_string("tests/everything.correct")
            .unwrap()
            .into_bytes();

        assert_eq!(Hext::new().parse(&to_parse).unwrap(), cmp)
    }

    //## Failing Tests ##
    #[test]
    fn ftest_incompleteoctet() {
        let test = "~little-endian msb0\n4";

        assert_eq!(
            Hext::new().parse(&test).unwrap_err(),
            Error::IncompleteOctet
        );
    }

    #[test]
    fn ftest_invalidcharacter() {
        let test = "~little-endian msb0\nG";

        assert_eq!(
            Hext::new().parse(&test).unwrap_err(),
            Error::InvalidCharacter('G')
        );
    }

    #[test]
    fn ftest_unaligned_bit() {
        let test = "~little-endian msb0\n.1";
        let cmp = Error::UnalignedBits;

        assert_eq!(Hext::new().parse(&test).unwrap_err(), cmp);
    }

    #[test]
    fn ftest_unaligned_bit_then_byte() {
        let test = "~little-endian msb0\n.1 41";
        let cmp = Error::UnalignedBits;

        assert_eq!(Hext::new().parse(&test).unwrap_err(), cmp);
    }
}
