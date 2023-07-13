mod parser_combinator;

use parser_combinator::*;

fn main() {
    //TODO how to combine
    //pthen does not take 2 parameters, but a function
    //let h = move |input| { por!(pchar => 'H', pchar => 'h', input) };
    //h("Hello".into());

    let result = pthen!(pchar!('H'), pchar!('e'), "Hello".into());
    println!("{:?}", result);
}
