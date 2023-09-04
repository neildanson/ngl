use crate::{parser_combinator::Parser, untyped_language::pfun, web::templates::*};
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

pub async fn run() {
    let app = Router::new()
        .route("/greet", get(greet))
        .route("/code", post(code))
        .fallback_service(ServeDir::new("wwwroot")); //.with_state(counter);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn greet() -> impl IntoResponse {
    let template = HelloTemplate {
        name: "Clicked".to_string(),
        count: 1,
    };
    HtmlTemplate(template)
}

use serde::Deserialize;

#[derive(Deserialize)]
struct Code {
    code: String,
}

async fn code(Form(code): Form<Code>) -> impl IntoResponse {
    let parser = pfun();
    let start = std::time::Instant::now();
    let result = parser.parse(code.code.as_str().into());
    let end = std::time::Instant::now();

    let (result, color) = match result {
        Ok(result) => (format!("{:#?}", result), "green".to_string()),
        Err(err) => (err.to_string(), "red".to_string()),
    };

    let template = CodeTemplate {
        duration: format!("{:?}", end - start),
        result: result.to_string(),
        color,
    };
    HtmlTemplate(template)
}
