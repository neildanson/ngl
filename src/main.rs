mod parser_combinator;

use parser_combinator::*;

fn main() {
    let h = move |x| por!(pchar!('H'), pchar!('h'), x);

    let result = pthen!(h, pchar!('e'), "hello".into());
    println!("{:?}", result);
}
