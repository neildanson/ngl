use ngl::parser_combinator::*;
use ngl::untyped_language::*;

fn main() {
    let fun_binding = pfun();

    let start = std::time::Instant::now();

    let result = fun_binding.parse(
        "fun name(param: type, paramx: typex) {
            let x = 1;
            call(x, y);
        }"
        .into(),
    );

    let end = std::time::Instant::now();

    println!("{:#?} \n\nTook {:?}", result, (end - start));
}
