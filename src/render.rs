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
                    link rel="icon" href="/static/img/hoh2_logo.png";
                    link href="https://fonts.googleapis.com/css?family=Fira+Sans|Lora|Niconne|Open+Sans&display=swap&subset=cyrillic" rel="stylesheet";
                    link rel="stylesheet" href="/static/css/main.css";
                    @for path in self.stylesheets { link rel="stylesheet" href=(path); }
                    @for path in self.scripts { script src=(path) {} }
                }
                body {
                    header {
                        nav.site-nav {
                            .site-title {
                                img src="/static/img/hoh2_logo.png";
                                img src="/static/img/hoh2_title.png" alt=(SITE_NAME);
                            }
                            ul.site-dirs {
                                li.dir { a.site-dir href=(uri!(handlers::index)) { "Главная" } }
                                li.dir { a.site-dir href=(uri!(handlers::characters)) { "Персонажи" } }
                                li.dir { a.site-dir href="/forum" { "Форум" } }
                                @if let Some(some_account) = self.account {
                                    li.auth {
                                        a.site-dir.current-user href=(some_account.href()) { (some_account.nick) }
                                    }
                                    li.auth {
                                        form method="post" action=(uri!(handlers::logout)) {
                                            button.site-dir.logout type="submit" { "Выход" }
                                        }
                                    }
                                } @else {
                                    li.auth { a.site-dir.login href=(uri!(handlers::login_page)) { "Вход" } }
                                    li.auth { a.site-dir.signup href=(uri!(handlers::signup)) { "Регистрация" } }
                                }
                            }
                        }
                    }
                    main { (content) }
                }
            }
        }
    }
}
