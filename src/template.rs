use crate::db::{item::Item, list::List, prefill_item::PrefillItem};
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tera::{Context, Tera, context};

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
pub struct ListPage<'a> {
    items: Vec<Item>,
    list_id: i32,
    templates: &'a Tera,
}

impl<'a> ListPage<'a> {
    pub fn new(templates: &'a Tera, items: Vec<Item>, list_id: i32) -> Self {
        ListPage {
            items,
            list_id,
            templates,
        }
    }

    pub fn render(self) -> Response {
        let mut context = Context::new();

        context.insert("items", &self.items);
        context.insert("list_id", &self.list_id);

        match self.templates.render("list.html", &context) {
            Ok(page) => Html(page).into_response(),
            Err(_) => {
                NotFoundPage::new(self.templates, Some("list.html template failed to render"))
                    .render()
            }
        }
    }
}

#[derive(Debug)]
pub struct IndexPage<'a> {
    username: &'a str,
    lists: Vec<List>,
    templates: &'a Tera,
}

impl<'a> IndexPage<'a> {
    pub fn new(templates: &'a Tera, username: &'a str, lists: Vec<List>) -> Self {
        IndexPage {
            username,
            lists,
            templates,
        }
    }

    pub fn render(self) -> Response {
        let mut context = Context::new();
        context.insert("username", &self.username);
        context.insert("lists", &self.lists);
        match self.templates.render("index.html", &context) {
            Ok(page) => Html(page).into_response(),
            Err(_) => InternalServerErrorPage::new(self.templates).render(),
        }
    }
}

#[derive(Debug)]
pub struct ItemPage<'a> {
    item: Item,
    templates: &'a Tera,
}

impl<'a> ItemPage<'a> {
    pub fn new(templates: &'a Tera, item: Item) -> Self {
        ItemPage { item, templates }
    }

    pub fn render(self) -> Response {
        let mut context = Context::new();
        context.insert("item", &self.item);
        match self.templates.render("item.html", &context) {
            Ok(page) => Html(page).into_response(),
            Err(_) => InternalServerErrorPage::new(self.templates).render(),
        }
    }
}

#[derive(Debug)]
pub struct AddItemPage<'a> {
    prefill_items: Vec<PrefillItem>,
    list_id: String,
    templates: &'a Tera,
}

impl<'a> AddItemPage<'a> {
    pub fn new(templates: &'a Tera, prefill_items: Vec<PrefillItem>, list_id: String) -> Self {
        AddItemPage {
            prefill_items,
            list_id,
            templates,
        }
    }

    pub fn render(self) -> Response {
        let mut context = Context::new();

        context.insert("list_id", &self.list_id);
        context.insert("prefill_items", &self.prefill_items);

        println!("check3");

        match self.templates.render("add_item.html", &context) {
            Ok(page) => Html(page).into_response(),
            Err(_) => InternalServerErrorPage::new(self.templates).render(),
        }
    }
}
