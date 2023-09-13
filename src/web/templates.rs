use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

// See : https://djc.github.io/askama/template_syntax.html

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "hello.html")]
pub struct HelloTemplate {
    pub name: String,
    pub count: u32,
}

#[derive(Template)]
#[template(path = "code.html")]
pub struct CodeTemplate {
    pub duration: String,
    pub result: String,
    pub success: String,
    pub success_style: String,
    pub color: String,
}
