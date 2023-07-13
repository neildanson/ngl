mod parser_combinator;

fn main() {
    let result = parser_combinator::pchar('t', "true".into());

    println!("{:?}", result);
}
