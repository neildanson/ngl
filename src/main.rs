mod parser_combinator;

use parser_combinator::*;

fn main() {
    let parser = pthen!(
        poptional!(pchar!('\n')),
        pthen!(por!(pchar!('H'), pchar!('h')), pchar!('e'))
    );
    let result = parser("\nhello".into());
    println!("{:?}", result);
}
