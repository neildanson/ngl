#[derive(Debug, PartialEq)]
pub struct Token<T> {
    pub value: T,
    pub start: usize,
    pub length: usize,
}

impl<T> Token<T> {
    pub fn new(value: T, start: usize, length: usize) -> Self {
        Self {
            value,
            start,
            length,
        }
    }
}

impl<T: Clone> Clone for Token<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            start: self.start,
            length: self.length,
        }
    }
}
