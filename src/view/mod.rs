pub mod auth;
pub mod index;
pub mod pc;

use http::StatusCode;
use maud::{html, Markup, DOCTYPE};
use crate::ctx;
use crate::web::session::Session;

pub struct Page {
    pub status: StatusCode,
    pub redirect: Option<(usize, String)>,
    pub page_title: Option<String>,
    pub stylesheets: Vec<String>,
    pub scripts: Vec<String>,
    pub content: Option<Markup>,
}

impl Page {
    pub fn untitled() -> Self {
        Self {
            status: StatusCode::OK,
            redirect: None,
            page_title: None,
            stylesheets: Vec::new(),
            scripts: Vec::new(),
            content: None
        }
    }

    pub fn titled<T>(title: T) -> Self where T: Into<String> {
        Self {
            status: StatusCode::OK,
            redirect: None,
            page_title: Some(title.into()),
            stylesheets: Vec::new(),
            scripts: Vec::new(),
            content: None
        }
    }

    pub fn status<T>(mut self, status: T) -> Self where T: Into<StatusCode> {
        self.status = status.into();
        self
    }

    pub fn redirect<T>(mut self, delay: usize, location: T) -> Self where T: Into<String> {
        self.redirect = Some((delay, location.into()));
        self
    }

    pub fn stylesheet<T>(mut self, path: T) -> Self where T: Into<String> {
        self.stylesheets.push(path.into());
        self
    }

    pub fn script<T>(mut self, path: T) -> Self where T: Into<String> {
        self.scripts.push(path.into());
        self
    }

    pub fn content(mut self, markup: Markup) -> Self {
        self.content = Some(markup);
        self
    }

    pub fn render(self, session: Option<&Session>) -> impl warp::Reply {
        let config = &ctx().site_config;
        let output = html! {
            (DOCTYPE);
            html lang="ru" {
                head {
                    @if let Some((delay, location)) = self.redirect {
                        meta http-equiv="refresh" content=(format!("{};{}", delay, location));
                    }
                    @if let Some(page_title) = self.page_title {
                        title { (page_title) " – " (config.title) }
                    } @else {
                        title { (config.title) }
                    }
                    @if let Some(logo_small) = &config.logo_small {
                        link rel="icon" href=(logo_small);
                    }
                    link rel="stylesheet" href="/assets/main.css";
                    @for href in self.stylesheets {
                        link rel="stylesheet" href=(href);
                    }
                    @for href in self.scripts {
                        script src=(href) {}
                    }
                }
                body {
                    header {
                        nav.site-nav {
                            .site-title {
                                @if let Some(logo) = &config.logo {
                                    img src=(logo) alt=(config.title);
                                } @else {
                                    (config.title)
                                }
                            }
                            ul.site-dirs {
                                @for (item_title, item_href) in &config.menu {
                                    li.dir { a.site_dir href=(item_href) { (item_title) } }
                                }
                                li.separator { "\u{00B7}" }
                                @if let Some(session) = session {
                                    li.auth {
                                        a.current-user href=(format!("/users/{}", session.account.nick)) { (session.account.nick) }
                                    }
                                    li.auth {
                                        form method="post" action="/logout" {
                                            button.site-dir.logout type="submit" { "[Выход]" }
                                        }
                                    }
                                } @else {
                                    li.auth { a.site-dir.login href="/login" { "Вход" } }
                                    li.auth { a.site-dir.signup href="/register" { "Регистрация" } }
                                }
                            }
                        }
                    }
                    @if let Some(content) = self.content {
                        main { (content) }
                    }
                }
            }
        };
        warp::reply::with_status(warp::reply::html(output.into_string()), self.status)
    }
}

pub struct SpecialPage {
    status: StatusCode,
    redirect: Option<String>,
    message: Option<Markup>,
    error: Option<String>,
}

impl SpecialPage {
    pub fn new<T>(status: T) -> Self where T: Into<StatusCode> {
        Self {
            status: status.into(),
            redirect: None,
            message: None,
            error: None,
        }
    }

    pub fn redirect<T>(mut self, redirect: T) -> Self where T: Into<String> {
        self.redirect = Some(redirect.into());
        self
    }

    pub fn message<T>(mut self, message: T) -> Self where T: Into<Markup> {
        self.message = Some(message.into());
        self
    }

    pub fn error<T>(mut self, error: T) -> Self where T: Into<String>  {
        self.error = Some(error.into());
        self
    }

    pub fn render(self) -> impl warp::Reply {
        let config = &ctx().site_config;
        let success = self.status.is_success();
        let error = self.status.is_client_error() || self.status.is_server_error();
        let info = !success && !error;
        let output = html! {
            (DOCTYPE);
            html lang="ru" {
                head {
                    @if let Some(location) = &self.redirect {
                        meta http-equiv="refresh" content=(format!("3;{}", location));
                    }
                    title { (config.title) }
                    @if let Some(logo_small) = &config.logo_small {
                        link rel="icon" href=(logo_small);
                    }
                    link rel="stylesheet" href="/assets/main.css";
                }
                body {
                    main.special {
                        .message-box {
                            .side {
                                @if let Some(logo_small) = &config.logo_small {
                                    img src=(logo_small);
                                }
                            }
                            .message.info[info].success[success].error[error] {
                                @if let Some(message) = &self.message {
                                    (message);
                                }
                                @if let Some(error) = &self.error {
                                    pre { (error) }
                                }
                            }
                            .side {}
                        }
                        @if self.redirect.is_some() {
                            .spinner {
                                .bounce1 {}
                                .bounce2 {}
                                .bounce3 {}
                            }
                        }
                    }
                }
            }
        };
        Box::new(warp::reply::with_status(warp::reply::html(output.into_string()), self.status))
    }
}
