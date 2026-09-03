use crate::db::item::Item;
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

#[derive(Debug)]
pub struct LoginPage<'a> {
    forward_to: String,
    flash: Option<String>,
    templates: &'a Tera,
}

impl<'a> LoginPage<'a> {
    pub fn new(templates: &'a Tera, forward_to: Option<&String>, flash: Option<String>) -> Self {
        let forward_to = forward_to.cloned().unwrap_or_else(|| "/".to_string());
        // let flash = match flash {
        //     Some(val) => Some(val.to_string()),
        //     None => None,
        // };

        LoginPage {
            forward_to: forward_to,
            flash: flash,
            templates: templates,
        }
    }

    pub fn render(self) -> Response {
        let mut context = Context::new();

        context.insert("forward_to", &self.forward_to);

        if let Some(flash) = self.flash {
            context.insert("flash", &flash);
        }

        match self.templates.render("login.html", &context) {
            Ok(page) => Html(page).into_response(),
            Err(_) => {
                NotFoundPage::new(self.templates, Some("login.html template failed to render"))
                    .render()
            }
        }
    }
}

#[derive(Debug)]
pub struct InternalServerErrorPage<'a> {
    templates: &'a Tera,
}

impl<'a> InternalServerErrorPage<'a> {
    pub fn new(templates: &'a Tera) -> Self {
        InternalServerErrorPage {
            templates: templates,
        }
    }

    pub fn render(self) -> Response {
        let context = Context::new();
        match self
            .templates
            .render("internal_server_error.html", &context)
        {
            Ok(page) => Html(page).into_response(),
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_server_error.html failed to render",
                )
                    .into_response();
            }
        }
    }
}

#[derive(Debug)]
pub struct List<'a> {
    items: Vec<Item>,
    templates: &'a Tera,
}

impl<'a> List<'a> {
    pub fn new(templates: &'a Tera, items: Vec<Item>) -> Self {
        List { items, templates }
    }

    pub fn render(self) -> Response {
        let mut context = Context::new();

        context.insert("items", &self.items);

        match self.templates.render("list.html", &context) {
            Ok(page) => Html(page).into_response(),
            Err(_) => {
                NotFoundPage::new(self.templates, Some("list.html template failed to render"))
                    .render()
            }
        }
    }
}
