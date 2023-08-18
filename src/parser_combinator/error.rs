use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::{Add, RangeInclusive},
};

#[derive(Clone, PartialEq, Debug)]
pub enum Expected<'a> {
    Char(char),
    String(&'a str),
    Any(Vec<char>),
    Range(RangeInclusive<char>),
    Or(Box<Expected<'a>>, Box<Expected<'a>>),
    And(Box<Expected<'a>>, Box<Expected<'a>>),
}

impl<'a> From<char> for Expected<'a> {
    fn from(c: char) -> Self {
        Expected::Char(c)
    }
}

impl<'a> From<&'a str> for Expected<'a> {
    fn from(s: &'a str) -> Self {
        Expected::String(s)
    }
}

impl<'a> From<RangeInclusive<char>> for Expected<'a> {
    fn from(r: RangeInclusive<char>) -> Self {
        Expected::Range(r)
    }
}

impl<'a> From<&[char]> for Expected<'a> {
    fn from(s: &[char]) -> Self {
        Expected::Any(s.to_vec())
    }
}

impl<'a, const N: usize> From<[char; N]> for Expected<'a> {
    fn from(s: [char; N]) -> Self {
        Expected::Any(s.to_vec())
    }
}

impl<'a> Add for Expected<'a> {
    type Output = Expected<'a>;
    fn add(self, rhs: Self) -> Self::Output {
        Expected::Or(Box::new(self), Box::new(rhs))
    }
}

impl<'a> Display for Expected<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Expected::Char(c) => write!(f, "'{}'", c),
            Expected::String(s) => write!(f, "'{}'", s),
            Expected::Range(range) => {
                write!(f, "between '{}' and '{}'", range.start(), range.end())
            }
            Expected::Or(lhs, rhs) => write!(f, "{} or {}", lhs, rhs),
            Expected::And(lhs, rhs) => write!(f, "{} and {}", lhs, rhs),
            Expected::Any(chars) => {
                let length = chars.len();
                let error = if length >= 2 {
                    let first = chars
                        .iter()
                        .take(length - 1)
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    first + " or " + &chars.last().unwrap().to_string()
                } else if length == 1 {
                    chars.first().unwrap().to_string()
                } else {
                    "".to_string() //TODO - this should never happen
                };
                write!(f, "{}", error)
            }
        }
    }
}

#[derive(PartialEq)]
pub struct Error<'a> {
    pub expected: Expected<'a>,
    pub actual: String,
    pub position: usize,
    pub line_number: usize,
    pub line_position: usize,
}

impl<'a> Error<'a> {
    pub fn new(
        expected: Expected<'a>,
        actual: String,
        position: usize,
        line_number: usize,
        line_position: usize,
    ) -> Self {
        Self {
            expected,
            actual,
            position,
            line_number,
            line_position,
        }
    }
    fn format_error(&self) -> String {
        format!(
            "Expected {} but got {} at line: {}, column: {}",
            self.expected,
            self.actual,
            self.line_number + 1,
            self.line_position + 1
        )
    }
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.format_error())
    }
}

impl<'a> Debug for Error<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.format_error())
    }
}

impl<'a> Add for Error<'a> {
    type Output = Error<'a>;

    fn add(self, other: Error<'a>) -> Self::Output {
        let expected = self.expected + other.expected;
        let actual = other.actual.clone();
        let position = other.position;
        let line_number = other.line_number;
        let line_position = other.line_position;
        Error::new(expected, actual, position, line_number, line_position)
    }
}
