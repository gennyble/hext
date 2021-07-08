use std::error::Error as ErrorTrait;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
	NoHeader,
	InvalidHeader(InvalidHeaderKind),

	IncompleteOctet,

	InvalidDecimal(String),
	InvalidSignedDecimal(String),
	InvalidUnsignedDecimal(String),
	InvalidBitness(String),

	InvalidCharacter(char),

	InvalidEscape(char),
	UnclosedStringLiteral,

	GarbageCharacterInBitstream,

	UnalignedBits,
}

impl ErrorTrait for Error {
	fn source(&self) -> Option<&(dyn ErrorTrait + 'static)> {
		None
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Error::NoHeader => write!(f, "The file must start with a header"),
			Error::InvalidHeader(kind) => write!(f, "{}", kind),
			Error::InvalidCharacter(c) => write!(f, "'{}' is not valid base16", c),
			Error::InvalidEscape(c) => write!(f, "\\{} is not a valid escape code", c),
			Error::UnclosedStringLiteral => {
				write!(
					f,
					"The line or file ended in an unterminated string literal"
				)
			}
			Error::IncompleteOctet => write!(f, "Octet was not complete"),
			Error::GarbageCharacterInBitstream => write!(
				f,
				"Periods to indicate binary data must be directly followed by that data"
			),
			Error::UnalignedBits => write!(f, "Not enough bits to form an octet"),
			Error::InvalidDecimal(string) => write!(f, "'{}' is not valid decimal", string),
			Error::InvalidSignedDecimal(value) => {
				write!(f, "'{}' is not valid signed decimal", value)
			}
			Error::InvalidUnsignedDecimal(value) => {
				write!(f, "'{}' is not valid unsigned decimal", value)
			}
			Error::InvalidBitness(bitness) => write!(
				f,
				"'{}' is not a valid width. Valid widths are 8, 16, 32, and 64",
				bitness
			),
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum InvalidHeaderKind {
	TwoBitOrder,
	TwoByteOrder,
	NoBitOrder,
	NoByteOrder,
	InvalidProperty(String),
}

impl fmt::Display for InvalidHeaderKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			InvalidHeaderKind::TwoBitOrder => write!(f, "You may only specify the bit order once"),
			InvalidHeaderKind::TwoByteOrder => {
				write!(f, "You may only specify the byte order once")
			}
			InvalidHeaderKind::NoBitOrder => write!(f, "You must specify a bit order"),
			InvalidHeaderKind::NoByteOrder => write!(f, "You must specify a byte order"),
			InvalidHeaderKind::InvalidProperty(property) => {
				write!(f, "'{}' is not a valid file property", property)
			}
		}
	}
}

impl Into<Error> for InvalidHeaderKind {
	fn into(self) -> Error {
		Error::InvalidHeader(self)
	}
}
