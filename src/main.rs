use crate::parser_combinator::Parser;

mod parser_combinator;

fn main() {
    let true_parser = parser_combinator::pstring("true");
    //let bool_parser = parser_combinator::pmap(true_parser, &|_| true);
    let result = true_parser.parse("true".into());

    println!("{:?}", result);
}
