mod parser_combinator;

use parser_combinator::*;

fn main() {
    let parser = pthen(
        poptional(pchar('\n')),
        pthen(por(pchar('H'), pchar('h')), pstring("ello")),
    );

    let choice = pchoice!(pchar('a'), pchar('b'), pchar('c'));
    let any = pany!('a', 'b', 'c');

    println!("{:?}", choice("d".into()));

    let result = parser("\nhel1o".into());

    println!("{:?}", result);
}
