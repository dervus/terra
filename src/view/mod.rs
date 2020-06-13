use maud::{html, Markup};
use crate::campaign::Campaign;

pub mod character;
pub use character::*;

pub fn campaign_page(campaign: &Campaign) -> Markup {
    html! {
        article {
            (maud::PreEscaped(&campaign.description))
        }
    }
}

pub fn roles_page(campaign: &Campaign) -> Markup {
    html! {
        h1 { "Роли" }
        ul.blocks {
            @for block in &campaign.blocks {
                li {
                    span { (block.name) }
                    ul {
                        @for role_id in &block.roles {
                            @if let Some(role) = campaign.roles.get(role_id) {
                                li {
                                    (&role.name);
                                    a href=(format!("/characters/new/{}", role_id)) {
                                        "Создать заявку"
                                    }
                                }
                            } @else {
                                li { "[missing: " (role_id) "]" }
                            }
                        }
                    }
                }
            }
        }
        .links {
            a href="/characters/new" { "Создать особую заявку" }
        }
    }
}


pub fn forum_page() -> Markup {
    html! {
        .alert {
            "Здесь будет форум. Но сейчас его нет."
        }
    }
}

fn field(ftype: &str, name: &str, label: &str) -> Markup {
    html! {
        div.label { label for=(name) { (label) } }
        div.input { input type=(ftype) name=(name); }
    }
}

pub fn signup() -> Markup {
    html! {
        .alert {
            "Регистрация в настоящее время отключена"
        }
        form.auth method="post" {
            fieldset disabled? {
                (field("text", "nick", "Ник"));
                (field("email", "email", "Email"));
                (field("password", "password", "Пароль"));
                (field("password", "password_confirm", "Повтор пароля"));
            }
            .form-controls {
                button type="submit" disabled? { "Регистрация" }
            }
        }
    }
}

pub fn login() -> Markup {
    html! {
        form.auth method="post" {
            fieldset {
                (field("text", "name", "Ник или email"));
                (field("password", "password", "Пароль"));
            }
            .form-controls {
                button type="submit" { "Вход" }
            }
        }
    }
}

pub fn redirect_page(location: &str) -> Markup {
    html! {
        .redirect-page {
            .alert {
                p {
                    "Пара мгновений и...";
                }
                p {
                    "Если ничего не происходит, то ";
                    a href=(location) { "нажмите здесь" }
                    ".";
                }
            }
        }
    }
}
