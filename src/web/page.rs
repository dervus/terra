use time::Duration;
use http::StatusCode;
use cookie::{Cookie, SameSite};
use maud::{html, Markup, DOCTYPE};
use crate::db::AccountInfo;
use crate::view;

const SITE_NAME: &'static str = "Skyland Next";

pub enum Session {
    Unauthed,
    JustLoggedOut,
    LoggedIn(String, AccountInfo),
}

pub struct Page {
    pub session: Session,
    pub status: StatusCode,
    pub redirect: Option<(usize, String)>,
    pub title: Option<String>,
    pub stylesheets: Vec<String>,
    pub scripts: Vec<String>,
    pub content: Option<Markup>,
}

impl Page {
    pub fn new() -> Self {
        Self {
            session: Session::Unauthed,
            status: StatusCode::OK,
            title: None,
            redirect: None,
            stylesheets: Vec::new(),
            scripts: Vec::new(),
            content: None
        }
    }

    pub fn session(mut self, session: Session) -> Self {
        self.session = session;
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
                                @if let Session::LoggedIn(_, account_info) = self.session {
                                    li.auth {
                                        span.current-user { (account_info.nick) }
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

impl warp::Reply for Page {
    fn into_response(self) -> warp::reply::Response {
        let mut res = http::Response::builder()
            .status(self.status)
            .header(http::header::CONTENT_TYPE, http::header::HeaderValue::from_static("text/html; charset=utf-8"));
        
        res = match &self.session {
            Session::LoggedIn(session_key, _) =>
                res.header(http::header::SET_COOKIE, update_session_cookie(session_key.clone()).to_string()),
            Session::JustLoggedOut =>
                res.header(http::header::SET_COOKIE, remove_session_cookie().to_string()),
            Session::Unauthed =>
                res,
        };
            
        res.body(self.into_markup().into_string().into()).unwrap()
    }
}

fn update_session_cookie(session_key: String) -> Cookie<'static> {
    Cookie::build("session", session_key)
        .path("/")
        .http_only(true)
        .secure(!cfg!(debug_assertions))
        .same_site(SameSite::Strict)
        .max_age(Duration::days(90))
        .finish()
}
    
fn remove_session_cookie() -> Cookie<'static> {
    Cookie::build("session", "")
        .path("/")
        .http_only(true)
        .secure(!cfg!(debug_assertions))
        .same_site(SameSite::Strict)
        .max_age(Duration::zero())
        .finish()
}
