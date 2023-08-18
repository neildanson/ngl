use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::Add,
};

#[derive(Clone, PartialEq)]
pub enum Expected {
    Char(char),
    String(String),
    Range(char, char),
}

impl Add for Expected {
    type Output = Expected;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Expected::Char(c1), Expected::Char(c2)) => Expected::String(format!("{}{}", c1, c2)),
            (Expected::Char(c1), Expected::String(s2)) => Expected::String(format!("{}{}", c1, s2)),
            (Expected::String(s1), Expected::Char(c2)) => Expected::String(format!("{}{}", s1, c2)),
            (Expected::String(s1), Expected::String(s2)) => {
                Expected::String(format!("{}{}", s1, s2))
            }
            (Expected::Char(c1), Expected::Range(start, end)) => {
                Expected::String(format!("{}{}-{}", c1, start, end))
            }
            (Expected::Range(start, end), Expected::Char(c2)) => {
                Expected::String(format!("{}-{}{}", start, end, c2))
            }
            (Expected::String(s1), Expected::Range(start, end)) => {
                Expected::String(format!("{}{}-{}", s1, start, end))
            }
            (Expected::Range(start, end), Expected::String(s2)) => {
                Expected::String(format!("{}-{}{}", start, end, s2))
            }
            (Expected::Range(start1, end1), Expected::Range(start2, end2)) => {
                Expected::String(format!("{}-{}{}-{}", start1, end1, start2, end2))
            }
        }
    }
}

impl Display for Expected {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Expected::Char(c) => write!(f, "'{}'", c),
            Expected::String(s) => write!(f, "'{}'", s),
            Expected::Range(start, end) => write!(f, "between '{}' and '{}'", start, end),
        }
    }
}

#[derive(PartialEq)]
pub struct Error {
    pub expected: Expected,
    pub actual: String,
    pub position: usize,
    pub line_number: usize,
    pub line_position: usize,
}

impl Error {
    pub fn new(
        expected: Expected,
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
            "Expected '{}' but got '{}' at line: {}, column: {}",
            self.expected,
            self.actual,
            self.line_number + 1,
            self.line_position + 1
        )
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.format_error())
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.format_error())
    }
}

impl Add for Error {
    type Output = Error;

    fn add(self, other: Error) -> Self::Output {
        let expected = self.expected + other.expected;
        let actual = other.actual.clone();
        let position = other.position;
        let line_number = other.line_number;
        let line_position = other.line_position;
        Error::new(expected, actual, position, line_number, line_position)
    }
}
