mod parser_combinator;

use parser_combinator::*;

fn main() {
    let parser = pthen!(por!(pchar!('H'), pchar!('h')), pchar!('e'));
    let result = parser("hello".into());
    println!("{:?}", result);
}
