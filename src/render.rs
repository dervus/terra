use maud::{html, Markup, DOCTYPE};
use crate::db::AccountInfo;
use crate::handlers;

const SITE_NAME: &'static str = "History of Heroes 2";

pub struct Page<'a> {
    title: Option<&'a str>,
    account: Option<&'a AccountInfo>,
    stylesheets: Vec<&'a str>,
    scripts: Vec<&'a str>,
}

impl<'a> Page<'a> {
    pub fn new() -> Self {
        Self {
            title: None,
            account: None,
            stylesheets: Vec::new(),
            scripts: Vec::new()
        }
    }
    
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn account(mut self, account: Option<&'a AccountInfo>) -> Self {
        self.account = account;
        self
    }

    pub fn stylesheet(mut self, path: &'a str) -> Self {
        self.stylesheets.push(path);
        self
    }

    pub fn script(mut self, path: &'a str) -> Self {
        self.scripts.push(path);
        self
    }

    pub fn render(self, content: Markup) -> Markup {
        html! {
            (DOCTYPE);
            html lang="ru" {
                head {
                    @if let Some(some_title) = self.title {
                        title { (some_title) " – " (SITE_NAME) }
                    } @else {
                        title { (SITE_NAME) }
                    }
                    link rel="icon" href="/static/gonglhead.png";
                    link rel="stylesheet" href="/static/main.css";
                    @for path in self.stylesheets { link rel="stylesheet" href=(path); }
                    @for path in self.scripts { script src=(path) {} }
                }
                body {
                    header {
                        nav.site-nav {
                            .site-logo {
                                a href="/" {
                                    img src="/static/hoh2logo.png";
                                    img src="/static/hoh2logotext.png" alt=(SITE_NAME);
                                }
                            }
                            ul.site-dirs {
                                li.dir { a href=(uri!(handlers::index)) { "Главная" } }
                                li.dir { a href=(uri!(handlers::characters)) { "Персонажи" } }
                                @if let Some(some_account) = self.account {
                                    li.auth {
                                        a.current-user href=(some_account.href()) { (some_account.nick) }
                                    }
                                    li.auth {
                                        form method="post" action=(uri!(handlers::logout)) {
                                            button.logout type="submit" { "Выход" }
                                        }
                                    }
                                } @else {
                                    li.auth { a.login href=(uri!(handlers::login_page)) { "Вход" } }
                                    li.auth { a.signup href=(uri!(handlers::signup)) { "Регистрация" } }
                                }
                            }
                        }
                    }
                    main { (content) }
                    footer { (crate::util::random_footnote()) }
                }
            }
        }
    }
}
