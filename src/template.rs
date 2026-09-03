use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tera::{Context, Tera};

#[derive(Debug)]
pub struct NotFoundPage<'a> {
    explanation: Option<String>,
    templates: &'a Tera,
}

impl<'a> NotFoundPage<'a> {
    pub fn new(templates: &'a Tera, explanation: Option<&str>) -> Self {
        if let Some(explanation) = explanation {
            NotFoundPage {
                explanation: Some(explanation.to_string()),
                templates: templates,
            }
        } else {
            NotFoundPage {
                explanation: None,
                templates: templates,
            }
        }
    }

    pub fn render(self) -> Response {
        let mut context = Context::new();

        if let Some(explanation) = self.explanation {
            context.insert("explanation", &explanation);
        }

        match self.templates.render("not_found.html", &context) {
            Ok(page) => Html(page).into_response(),
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "not_found.html template missing",
                )
                    .into_response();
            }
        }
    }
}

#[derive(Debug)]
pub struct LogoutPage<'a> {
    message: String,
    templates: &'a Tera,
}

impl<'a> LogoutPage<'a> {
    pub fn new(templates: &'a Tera, message: &str) -> Self {
        LogoutPage {
            message: message.to_string(),
            templates: templates,
        }
    }

    pub fn render(self) -> Response {
        let mut context = Context::new();

        context.insert("message", &self.message);

        match self.templates.render("logout.html", &context) {
            Ok(page) => Html(page).into_response(),
            Err(_) => NotFoundPage::new(
                self.templates,
                Some("logout.html template failed to render."),
            )
            .render(),
        }
    }
}
