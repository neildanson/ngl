mod parser_combinator;

use parser_combinator::*;

fn pstring<'a>(test: &'a str, input: &'a str) -> ParseResult<'a, &'a str> {
    let mut cont = ContinuationState::new(input);
    for t in test.chars() {
        let parser = pchar!(t);
        let result = parser(cont);
        match result {
            Ok((_, new_cont)) => cont = new_cont,
            Err(err) => {
                //Todo construct error from input
                return Err(Error::new(
                    err.expected.to_string(),
                    err.actual.to_string(),
                    err.position,
                ));
            }
        }
    }

    //TODO construct cont from input
    return Ok((Token::new(test, 0, 5), cont));
}

fn main() {
    let parser = pthen!(
        poptional!(pchar!('\n')),
        pthen!(por!(pchar!('H'), pchar!('h')), pchar!('e'))
    );

    let result = pstring("ello", "hello1");
    println!("{:?}", result);

    let result = parser("\nhello".into());

    /*
    let rec sequence parserList =
        // define the "cons" function, which is a two parameter function
        let cons head tail = head::tail

        // lift it to Parser World
        let consP = lift2 cons

        // process the list of parsers recursively
        match parserList with
        | [] ->
            returnP []
        | head::tail ->
            consP head (sequence tail)
     */

    println!("{:?}", result);
}
