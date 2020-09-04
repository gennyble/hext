mod error;

use std::str::Chars;
use std::iter::Peekable;
use crate::error::Error;
use bitvec::prelude::*;

pub struct Options {
    pub allow_unaligned_bits: bool
}

impl Options {
    pub fn default() -> Options {
        Options {
            allow_unaligned_bits: false
        }
    }
}

pub fn to_bytes(raw: &str, options: &Options) -> Result<Vec<u8>, Error> {
    let mut ret: Vec<u8> = vec![];
    let mut bitopt: Option<BitVec<Msb0, u8>> = None;

    let mut chars = raw.chars().peekable();
    loop {
        // Advance iterator and ignore comments
        match chars.peek() {
            Some('#') => {
                chars.find(|&c| c == '\n');
                continue;
            },
            Some(_c) => (), 
            None => {
                if bitopt.is_some() {
                    let mut bitvec = bitopt.take().unwrap();

                    if !options.allow_unaligned_bits && bitvec.len() % 8 != 0{
                        // If the options disallow unaligned bits, and we have one, return an error
                        return Err(Error::UnalignedBits);
                    }

                    pad_bitvec(&mut bitvec);
                    ret.append(&mut bitvec.into_vec());
                }

                return Ok(ret);
            }
        }

        // Whitespce doesn't mater
        if chars.peek().unwrap().is_whitespace() {
            chars.next();
            continue;
        }

        match parse_byte(&mut chars) {
            Some(Ok(byte)) => {
                if bitopt.is_some() {
                    let mut bitvec = bitopt.take().unwrap();

                    if !options.allow_unaligned_bits && bitvec.len() % 8 != 0{
                        // If the options disallow unaligned bits, and we have one, return an error
                        return Err(Error::UnalignedBits);
                    }

                    pad_bitvec(&mut bitvec);
                    ret.append(&mut bitvec.into_vec());
                }

                ret.push(byte);
            },
            Some(Err(err)) => return Err(err),
            None => match parse_bits(&mut chars) {
                Some(Ok(mut bits)) => {
                    if bitopt.is_some() {
                        bitopt.as_mut().unwrap().append(&mut bits);
                    } else {
                        bitopt = Some(bits);
                    }
                },
                Some(Err(err)) => return Err(err),
                None => continue
            }
        }
    }
}

fn parse_byte(chars: &mut Peekable<Chars>) -> Option<Result<u8, Error>> {
    match chars.peek() {
        Some(c) if c.is_digit(16) => {
            let high = chars.next().unwrap().to_digit(16).unwrap() * 16;

            match chars.next() {
                Some(c) if c.is_digit(16) => {
                    let low = c.to_digit(16).unwrap();

                    Some(Ok((high + low) as u8))
                },
                Some(c) => Some(Err(Error::InvalidCharacter(c))),
                None => Some(Err(Error::IncompleteOctet))
            }
        },
        Some(c) if *c != '.' =>  Some(Err(Error::InvalidCharacter(*c))),
        _ => None
    }
}

fn parse_bits(chars: &mut Peekable<Chars>) -> Option<Result<BitVec<Msb0, u8>, Error>> {
    let mut bv: BitVec<Msb0, u8> = BitVec::new();

    match chars.peek() {
        Some('.') => {
            chars.next(); // Throwaway the dot

            loop {
                match chars.peek() {
                    Some('0') => {
                        chars.next();
                        bv.push(false);
                    },
                    Some('1') => {
                        chars.next();
                        bv.push(true);
                    },
                    Some(c) if c.is_whitespace() || *c == '#' => return Some(Ok(bv)),
                    Some(c) => return Some(Err(Error::InvalidCharacter(*c))),
                    None => return Some(Ok(bv))
                }
            }
        },
        Some(c) => Some(Err(Error::InvalidCharacter(*c))),
        None => None
    }
}

fn pad_bitvec(bv: &mut BitVec<Msb0, u8>) {
    while bv.len() % 8 != 0 {
        bv.insert(0, false);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    //## Bytes tests ##
    #[test]
    fn test_onebyte() {
        let test = "41";
        let cmp = vec![0x41];

        assert_eq!(to_bytes(&test, &Options::default()).unwrap(), cmp);
    }

    #[test]
    fn test_only_comment() {
        let test = "# Comment";

        assert_eq!(to_bytes(&test, &Options::default()).unwrap(), vec![]);
    }

    #[test]
    fn test_1byte_comment() {
        let test = "41 #A";
        let cmp = vec![0x41];

        assert_eq!(to_bytes(&test, &Options::default()).unwrap(), cmp);
    }

    #[test]
    fn test_byte_nospace_comment() {
        let test = "41#A";
        let cmp = vec![0x41];

        assert_eq!(to_bytes(&test, &Options::default()).unwrap(), cmp);
    }

    #[test]
    fn test_2byte_multiline() {
        let test = "41\n42";
        let cmp = vec![0x41, 0x42];

        assert_eq!(to_bytes(&test, &Options::default()).unwrap(), cmp);
    }

    #[test]
    fn test_2bytes_nospace() {
        let test = "4142";
        let cmp = vec![0x41, 0x42];

        assert_eq!(to_bytes(&test, &Options::default()).unwrap(), cmp);
    }

    //## Bit Tests ##
    #[test]
    fn test_8bits() {
        let test = ".01000001";
        let cmp = vec![0x41];

        assert_eq!(to_bytes(&test, &Options::default()).unwrap(), cmp);
    }

    #[test]
    fn test_8bit_comment() {
        let test = ".01000001 # A";
        let cmp = vec![0x41];

        assert_eq!(to_bytes(&test, &Options::default()).unwrap(), cmp)
    }

    #[test]
    fn test_8bit_nospace_comment() {
        let test = ".01000001#A";
        let cmp = vec![0x41];

        assert_eq!(to_bytes(&test, &Options::default()).unwrap(), cmp);
    }

    #[test]
    fn test_1bit() {
        let test = ".1";
        let cmp = vec![0x01];

        assert_eq!(to_bytes(&test, &Options { allow_unaligned_bits: true } ).unwrap(), cmp);
    }

    #[test]
    fn test_8bits_halved_space() {
        let test_space = ".0100 .0010";
        let cmp = vec![0x42];

        assert_eq!(to_bytes(&test_space, &Options::default()).unwrap(), cmp);
    }

    #[test]
    fn test_8bits_halved_line() {
        let test_line= ".0100\n.0010";
        let cmp = vec![0x42];

        assert_eq!(to_bytes(&test_line, &Options::default()).unwrap(), cmp);
    }

    #[test]
    fn test_8bits_halved_line_comments() {
        let test_line_comments = ".0100#Half of capital letter\n.0010 # B";
        let cmp = vec![0x42];

        assert_eq!(to_bytes(&test_line_comments, &Options::default()).unwrap(), cmp);
    }

    #[test]
    fn test_1bit_then_byte() {
        let test = ".1 41";
        let cmp = vec![0x01, 0x41];

        assert_eq!(to_bytes(&test, &Options { allow_unaligned_bits: true }).unwrap(), cmp);
    }

    //## Failing Tests ##
    #[test]
    fn ftest_incompleteoctet() {
        let test = "4";

        assert_eq!(to_bytes(&test, &Options::default()).unwrap_err(), Error::IncompleteOctet);
    }

    #[test]
    fn ftest_invalidcharacter() {
        let test = "G";

        assert_eq!(to_bytes(&test, &Options::default()).unwrap_err(), Error::InvalidCharacter('G'));
    }

    #[test]
    fn ftest_unaligned_bit() {
        let test = ".1";
        let cmp = Error::UnalignedBits;

        assert_eq!(to_bytes(&test, &Options::default()).unwrap_err(), cmp);
    }

    #[test]
    fn ftest_unaligned_bit_then_byte() {
        let test = ".1 41";
        let cmp = Error::UnalignedBits;

        assert_eq!(to_bytes(&test, &Options::default()).unwrap_err(), cmp);
    }
}
