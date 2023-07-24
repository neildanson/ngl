use std::fmt::{self, Debug, Display, Formatter};

#[derive(PartialEq)]
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
            self.expected, self.actual, self.line_number, self.line_position
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

fn combine_error(error1: Error, error2: Error) -> Error {
    let expected = error1.expected.clone() + " or " + &error2.expected;
    let actual = error2.actual.clone();
    let position = error2.position;
    let line_number = error2.line_number;
    let line_position = error2.line_position;
    Error::new(expected, actual, position, line_number, line_position)
}
