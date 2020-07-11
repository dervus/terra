use maud::{html, Markup};
use super::Page;
use crate::ctx;

const RECAPTCHA_SCRIPT_URL: &'static str = "https://www.google.com/recaptcha/api.js";

pub fn register() -> Page {
    Page::titled("Регистрация")
        .script(RECAPTCHA_SCRIPT_URL)
        .content(html! {
            form.auth-form method="post" {
                fieldset.auth-fields {
                    (field("text", "nick", "Ник"));
                    (field("email", "email", "Email"));
                    (field("password", "password", "Пароль"));
                    (field("password", "password_confirm", "Повтор пароля"));
                }
                (recaptcha());
                .form-controls {
                    button type="submit" { "Регистрация" }
                }
            }
        })
}

pub fn login(captcha: bool) -> Page {
    Page::titled("Вход")
        .script(RECAPTCHA_SCRIPT_URL)
        .content(html! {
            form.auth-form method="post" {
                fieldset.auth-fields {
                    (field("text", "nick_or_email", "Ник или email"));
                    (field("password", "password", "Пароль"));
                }
                @if captcha {
                    (recaptcha());
                }
                .form-controls {
                    button type="submit" { "Вход" }
                }
            }
        })
}

fn field(ftype: &str, name: &str, label: &str) -> Markup {
    html! {
        div.label-cell { label for=(name) { (label) } }
        div.input-cell { input id=(name) type=(ftype) name=(name); }
    }
}

fn recaptcha() -> Markup {
    let sitekey = &ctx().recaptcha.sitekey;
    html! {
        fieldset.g-recaptcha data-sitekey=(sitekey) data-theme="light" {}
    }
}
