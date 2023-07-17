mod parser_combinator;

use parser_combinator::*;

fn main() {
    let parser = pthen(
        poptional(pchar('\n')),
        pthen(por(pchar('H'), pchar('h')), pstring("ello")),
    );

    let result = parser("\nhel1o".into());

    println!("{:?}", result);
}
