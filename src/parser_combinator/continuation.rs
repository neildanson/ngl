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

    pub(crate) fn advance(self, abs: usize, new_line: bool) -> Self {
        let (line_number, line_position) = if new_line {
            (self.line_number + 1, 0)
        } else {
            (self.line_number, self.line_position + abs)
        };
        Self {
            remaining: &self.remaining[abs..],
            position: self.position + abs,
            line_number,
            line_position,
        }
    }
}

impl<'a> From<&'a str> for ContinuationState<'a> {
    fn from(input: &'a str) -> Self {
        Self::new(input)
    }
}
