use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::Add,
};

#[derive(PartialEq, Clone)]
pub struct Error {
    pub expected: String,
    pub actual: String,
    pub position: usize,
    pub line_number: usize,
    pub line_position: usize,
}

impl Error {
    pub fn new(
        expected: String,
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
        let expected = self.expected.clone() + " or " + &other.expected;
        let actual = other.actual.clone();
        let position = other.position;
        let line_number = other.line_number;
        let line_position = other.line_position;
        Error::new(expected, actual, position, line_number, line_position)
    }
}
