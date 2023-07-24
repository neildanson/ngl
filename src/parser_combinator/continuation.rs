#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ContinuationState<'a> {
    pub remaining: &'a str,
    pub position: usize,
    pub line_number: usize,
    pub line_position: usize,
}

impl<'a> ContinuationState<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            remaining: input,
            position: 0,
            line_number: 0,
            line_position: 0,
        }
    }

    pub(crate) fn advance(&self, abs: usize, line: usize) -> Self {
        Self {
            remaining: &self.remaining[abs..],
            position: self.position + abs,
            line_number: self.line_number + line,
            line_position: if line == 0 {
                self.line_position + abs
            } else {
                0
            },
        }
    }
}

impl<'a> From<&'a str> for ContinuationState<'a> {
    fn from(input: &'a str) -> Self {
        Self::new(input)
    }
}
