use std::rc::Rc;

#[derive(Debug)]
pub struct ParserState<'a> {
    remaining : &'a str,
    position : usize,
}

impl<'a> ParserState<'a> {
    fn new (input: &'a str) -> Self {
        Self {
            remaining: input,
            position: 0,
        }
    }


    fn advance(&self, n: usize) -> Self {
        Self {
            remaining: &self.remaining[n..],
            position: self.position + n,
        }
    }
}

impl<'a> From <&'a str> for ParserState<'a> {
    fn from(input: &'a str) -> Self {
        Self::new(input)
    }
}


pub trait Parser<'a> {
    type Output;
    fn parse(&self, input: ParserState<'a>) -> Result<(Self::Output, ParserState<'a>), String>;
}

pub struct ParserFn<'a, Output> {
    parser: Rc<dyn Fn (ParserState<'a>) -> Result<(Output, ParserState<'a>), String>>,
}

impl<'a,  Output> ParserFn<'a,  Output>  {
    pub fn new(parser: Rc<dyn Fn(ParserState<'a>) -> Result<(Output, ParserState<'a>), String>>) -> Self {
        Self { parser }
    }
}

impl <'a,  Output> Parser<'a> for ParserFn<'a,  Output>{
    type Output = Output;
    fn parse(&self, input: ParserState<'a>) -> Result<(Output, ParserState<'a>), String> {
        (self.parser)(input)
    }
}

pub fn pchar<'a>(c:char) -> ParserFn<'a, char> {
    ParserFn::new(Rc::new(move |input: ParserState| {
        let mut chars = input.remaining.chars();
        match chars.next() {
            Some(letter) if letter == c => {
                let parser_state = input.advance(1);
                Ok((c, parser_state))
            },
            _ => Err("No more input".to_string()),
        }
    }))
}