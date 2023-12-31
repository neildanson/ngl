use ngl::parser_combinator::*;
use ngl::untyped_language::*;

#[tokio::main]
async fn main() {
    let fun_binding = pfun();

    let start = std::time::Instant::now();

    let result = fun_binding.parse(
        "fun name(param: type, param_x: type_x) -> unit {
            let x = 1;
            let str = \"hello\";
            call(x, param, function(param_x,4));
            if true {
                call(1);
            } else {
                call(2);
            };

            for i = 0 .. 10 {     
                call(i);
            };
        }"
        .into(),
    );

    let end = std::time::Instant::now();

    println!("{:#?} \n\nTook {:?}", result, (end - start));
    ngl::web::app::run().await;
}
