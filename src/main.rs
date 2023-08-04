use ngl::parser_combinator::*;
use ngl::untyped_language::*;

fn main() {
    let fun_binding = pmany(pfun());

    let start = std::time::Instant::now();

    let result = fun_binding.parse(
        "fun name(param: type, param_x: type_x) -> unit {
            let x = 1;
            let str = \"hello\";
            call(x, param, function(param_x,4));
            
        }"
        .into(),
    );

    let end = std::time::Instant::now();

    println!("{:#?} \n\nTook {:?}", result, (end - start));
}
