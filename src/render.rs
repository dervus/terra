use maud::{html, Markup, DOCTYPE};
use crate::db::AccountInfo;
use crate::handlers;

const SITE_NAME: &'static str = "History of Heroes 2";

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SiteDir {
    Root,
    Campaigns,
    Characters,
}

pub struct Page<'a> {
    dir: SiteDir,
    title: Option<&'a str>,
    account: Option<&'a AccountInfo>,
    scripts: Vec<String>,
}

impl<'a> Page<'a> {
    pub fn new(dir: SiteDir) -> Self {
        Self { dir, title: None, account: None, scripts: Vec::new() }
    }
    
    pub fn root() -> Self { Self::new(SiteDir::Root) }
    pub fn campaigns() -> Self { Self::new(SiteDir::Campaigns) }
    pub fn characters() -> Self { Self::new(SiteDir::Characters) }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn account(mut self, account: Option<&'a AccountInfo>) -> Self {
        self.account = account;
        self
    }

    pub fn script<T: Into<String>>(mut self, path: T) -> Self {
        self.scripts.push(path.into());
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
                    @for path in self.scripts {
                        script src=(path) {}
                    }
                }
                body {
                    header {
                        nav.site-navigation {
                            .site-logo {
                                a href="/" {
                                    img src="/static/hoh2logo.png";
                                    img src="/static/hoh2logotext.png" alt=(SITE_NAME);
                                }
                            }
                            ul {
                                li { a.active[self.dir == SiteDir::Root] href=(uri!(handlers::index)) { "Главная" } }
                                li { a.active[self.dir == SiteDir::Campaigns] href=(uri!(handlers::campaigns)) { "Игры" } }
                                li { a.active[self.dir == SiteDir::Characters] href=(uri!(handlers::characters)) { "Персонажи" } }
                                li.separator {}
                                @if let Some(some_account) = self.account {
                                    li.current-account { a href=(some_account.href()) { (some_account.nick) } }
                                    li.logout {
                                        form method="post" action=(uri!(handlers::logout)) {
                                            button type="submit" { "Выход" }
                                        }
                                    }
                                } @else {
                                    li { a href=(uri!(handlers::login_page)) { "Вход" } }
                                    li { a href=(uri!(handlers::signup)) { "Регистрация" } }
                                }
                            }
                        }
                    }
                    main { (content) }
                    footer {}
                }
            }
        }
    }
}

pub fn field(ftype: &str, name: &str, label: &str) -> Markup {
    html! {
        div.label { label for=(name) { (label) } }
        div.input { input type=(ftype) name=(name); }
    }
}

pub fn capitalize<T: AsRef<str>>(input: T) -> String {
    let mut output = String::with_capacity(input.as_ref().len());
    for (index, character) in input.as_ref().to_owned().chars().enumerate() {
        if index == 0 {
            for upcase in character.to_uppercase() {
                output.push(upcase);
            }
        } else {
            output.push(character);
        }
    }
    output
}
