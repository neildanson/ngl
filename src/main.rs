mod parser_combinator;

use parser_combinator::*;

fn pthenwrapper<'a>(word: &'a str) -> ParseResult<'a, (char, char)> {
    pthen!(pchar => 'H', pchar => 'e', word.into())
}

fn main() {
    let result = pthenwrapper("Hello");

    println!("{:?}", result);
}
