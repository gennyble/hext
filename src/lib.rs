mod error;

use bitvec::prelude::*;
use crate::error::Error;

pub fn to_bytes(raw: &str) -> Result<Vec<u8>, Error> {
    let mut ret: Vec<u8> = vec![];

    for line in raw.lines() {
        ret.append(&mut process_bytes(if let Some(hash_index) = line.find('#') {
            &line[..hash_index]
        } else {
            &line
        })?);
    }

    Ok(ret)
}

fn process_bytes(raw: &str) -> Result<Vec<u8>, Error> {
    let mut ret: Vec<u8> = vec![];
    let mut bits: Option<BitBox> = None;

    let mut chars = raw.chars();
    let mut curr: char;

    loop {
        match chars.next() {
            Some(c) => curr = c,
            None => return Ok(ret)
        }

        if curr.is_whitespace() {
            continue;
        }

        if curr.is_digit(16) {
            match chars.next() {
                Some(c) if c.is_digit(16) => {
                    ret.push(((curr.to_digit(16).unwrap() * 16) + c.to_digit(16).unwrap()) as u8);
                    continue;
                },
                _ => {
                    return Err(Error::IncompleteOctet);
                }
            }
        } else {
            return Err(Error::InvalidCharacter(curr));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_onebyte() {
        let test = "41";
        let cmp = vec![0x41];

        assert_eq!(to_bytes(&test).unwrap(), cmp);
    }

    #[test]
    fn test_only_comment() {
        let test = "# Comment";

        assert_eq!(to_bytes(&test).unwrap(), vec![]);
    }

    #[test]
    fn test_1byte_comment() {
        let test = "41 #A";
        let cmp = vec![0x41];

        assert_eq!(to_bytes(&test).unwrap(), cmp);
    }

    #[test]
    fn test_2byte_multiline() {
        let test = "41\n42";
        let cmp = vec![0x41, 0x42];

        assert_eq!(to_bytes(&test).unwrap(), cmp);
    }

    #[test]
    fn test_2bytes_nospace() {
        let test = "4142";
        let cmp = vec![0x41, 0x42];

        assert_eq!(to_bytes(&test).unwrap(), cmp);
    }

    #[test]
    fn test_incompleteoctet() {
        let test = "4";

        assert_eq!(to_bytes(&test).unwrap_err(), Error::IncompleteOctet);
    }

    #[test]
    fn test_invalidcharacter() {
        let test = "G";

        assert_eq!(to_bytes(&test).unwrap_err(), Error::InvalidCharacter('G'));
    }
}
