use ngl::parser_combinator::*;
use ngl::untyped_language::*;

#[derive(Clone, Debug)]
enum Value {
    Number(i32),
    Bool(bool),
}

fn main() {
    let fun_binding = pfun();

    let result = fun_binding.parse(
        "fun name(param: type, paramx: typex) {
            let x = 1;
            let y = 2;
        }"
        .into(),
    );

    //let result = let_binding.parse("let x = true;".into());

    println!("{:#?}", result);
}
