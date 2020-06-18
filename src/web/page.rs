use http::StatusCode;
use maud::{html, Markup, DOCTYPE};
use super::session::WithSession;
use crate::db::account::Account;
use crate::view;

const SITE_NAME: &'static str = "Terra";

pub struct Page {
    pub account_info: Option<AccountInfo>,
    pub status: StatusCode,
    pub redirect: Option<(usize, String)>,
    pub title: Option<String>,
    pub stylesheets: Vec<String>,
    pub scripts: Vec<String>,
    pub content: Option<Markup>,
}

struct AccountInfo {
    id: i32,
    nick: String,
    access_level: i16,
}

impl Page {
    pub fn new() -> Self {
        Self {
            account_info: None,
            status: StatusCode::OK,
            title: None,
            redirect: None,
            stylesheets: Vec::new(),
            scripts: Vec::new(),
            content: None
        }
    }

    pub fn account<T>(mut self, account: Option<T>) -> Self where T: AsRef<Account> {
        self.account_info = account.map(|a| a.as_ref()).map(|a| AccountInfo {
            id: a.account_id,
            nick: a.nick.clone(),
            access_level: a.access_level,
        });
        self
    }

    pub fn status<T>(mut self, status: T) -> Self where T: Into<StatusCode> {
        self.status = status.into();
        self
    }

    pub fn title<T>(mut self, title: T) -> Self where T: Into<String> {
        self.title = Some(title.into());
        self
    }

    pub fn redirect<T>(mut self, delay: usize, location: T) -> Self where T: Into<String> {
        let location = location.into();
        let markup = view::redirect_page(&location);
        self.redirect = Some((delay, location));
        self.content(markup)
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

    pub fn wrap(self, ws: WithSession) -> WithSession {
        ws.with(self)
    }

    pub fn into_markup(self) -> Markup {
        html! {
            (DOCTYPE);
            html lang="ru" {
                head {
                    @if let Some((delay, location)) = self.redirect {
                        meta http-equiv="refresh" content=(format!("{};{}", delay, location));
                    }
                    @if let Some(some_title) = self.title {
                        title { (some_title) " – " (SITE_NAME) }
                    } @else {
                        title { (SITE_NAME) }
                    }
                    link rel="icon" href="/static/img/slnext-logo-small.png";
                    link href="https://fonts.googleapis.com/css?family=Fira+Sans|Lora|Niconne|Open+Sans&display=swap&subset=cyrillic" rel="stylesheet";
                    link rel="stylesheet" href="/static/css/main.css";
                    @for path in self.stylesheets { link rel="stylesheet" href=(path); }
                    script src="https://cdn.jsdelivr.net/npm/kefir@3/dist/kefir.min.js" {}
                    @for path in self.scripts { script src=(path) {} }
                }
                body {
                    header {
                        nav.site-nav {
                            .site-title {
                                img src="/static/img/slnext-logo.png" alt=(SITE_NAME);
                            }
                            ul.site-dirs {
                                li.dir { a.site-dir href="/" { "Об игре" } }
                                li.dir { a.site-dir href="/roles" { "Роли" } }
                                li.dir { a.site-dir href="/characters" { "Персонажи" } }
                                li.dir { a.site-dir href="/forum" { "Форум" } }
                                li { "\u{00B7}" }
                                @if let Some(info) = self.account_info {
                                    li.auth {
                                        span.current-user { (info.nick) }
                                    }
                                    li.auth {
                                        form method="post" action="/logout" {
                                            button.site-dir.logout type="submit" { "[Выход]" }
                                        }
                                    }
                                } @else {
                                    li.auth { a.site-dir.login href="/login" { "Вход" } }
                                    li.auth { a.site-dir.signup href="/signup" { "Регистрация" } }
                                }
                            }
                        }
                    }
                    main {
                        @if let Some(content) = self.content {
                            (content)
                        }
                    }
                }
            }
        }
    }
}
