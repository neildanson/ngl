mod parser_combinator;

use parser_combinator::*;

fn main() {
    let any_number = pany!('0', '1', '2', '3', '4', '5', '6', '7', '8', '9');
    let many_numbers = pmany(any_number);
    let number_parser = pthen(poptional(pchar('-')), many_numbers);

    let to_number = pmap(
        number_parser,
        move |(negate, value): (Option<char>, Vec<Token<char>>)| {
            let string: String = value.into_iter().map(|c| c.value).collect();
            let number = string.parse::<i32>().unwrap();
            match negate {
                Some(_) => -number,
                None => number,
            }
        },
    );

    let result = to_number("123".into());

    println!("{:?}", result);
}
