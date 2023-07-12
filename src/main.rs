mod parser_combinator;
use parser_combinator::Parser;

fn main() {
    let h_parser = parser_combinator::pchar('H');

    let result = h_parser.parse("".into());

    println!("{:?}", result);
}
