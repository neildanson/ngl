mod parser_combinator;
use parser_combinator::Parser;

fn main() {
    let string_parser = parser_combinator::pchar('H');

    let result = string_parser.parse("Hello".into());

    println!("{:?}", result);
}
