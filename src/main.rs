use ngl::parser_combinator::*;
use ngl::untyped_language::*;
use ngl::web::templates::*;

use axum::{routing::get, Router};
use axum::{
    extract::{self, State},
    response::IntoResponse,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;


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

    let app = Router::new()
        .route("/greet", get(greet))
        .fallback_service(ServeDir::new("wwwroot"))
        ;//.with_state(counter);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

}

pub async fn greet(
) -> impl IntoResponse {
    println!("Hello from greet");
    let template = HelloTemplate { name : "Clicked".to_string() , count : 1 };
    HtmlTemplate(template)
}
