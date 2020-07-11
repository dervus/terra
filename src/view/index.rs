use maud::{html, Markup};
use http::StatusCode;
use crate::framework::campaign::Campaign;
use super::{Page, SpecialPage};

pub fn campaign(campaign: &Campaign) -> Page {
    Page::titled(&campaign.name).content(html! {
        h1 { (campaign.name) }
        article {
            (maud::PreEscaped(&campaign.info))
        }
        h2 { "Роли" }
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
    })
}

pub fn forum() -> Page {
    Page::titled("Форум").content(html! {
        .alert {
            "Здесь будет форум. Но сейчас его нет."
        }
    })
}

pub fn redirect(status: StatusCode, location: &str) -> impl warp::Reply {
    SpecialPage::new(status)
        .redirect(location)
        .message(html! {
            p {
                "Пара мгновений и...";
            }
            p {
                "Если ничего не происходит, то ";
                a href=(location) { "нажмите здесь" }
                ".";
            }
        })
        .render()
}
