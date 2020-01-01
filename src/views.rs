use maud::{html, Markup, DOCTYPE};
use crate::system::Campaign;
use crate::render::*;

pub fn campaign(campaign: &Campaign) -> Markup {
    html! {
        // (data.description);
        // div.campaign-blocks {
        //     @for block in &data.blocks {
        //         h2 { (block.title) }
        //         ul {
        //             @for role in &block.roles {
        //                 li data-role-id=(role) { (role) }
        //             }
        //         }
        //     }
        // }
    }
}

pub fn character_form(campaign: &Campaign, selected_role: Option<&str>) -> Markup {
    html! {
        h1 { (campaign.title) }
        small { "Новый персонаж" }
        form {
            ul.selection {
                @for (race_id, race) in &campaign.races {
                    @let input_id = format!("race-{}", race_id);
                    li {
                        input id=(input_id) type="radio" name="race" value=(race_id);
                        label for=(input_id) { (capitalize(&race.name)) }
                    }
                }
            }
            ul.selection {
                @for (class_id, class) in &campaign.classes {
                    @let input_id = format!("class-{}", class_id);
                    li {
                        input id=(input_id) type="radio" name="class" value=(class_id);
                        label for=(input_id) { (capitalize(&class.name)) }
                    }
                }
            }
            ul.selection {
                @for (id, info) in &campaign.traits {
                    @let input_id = format!("trait-{}", id);
                    li {
                        input id=(input_id) type="checkbox" name="trait" value=(id);
                        label for=(input_id) { (capitalize(&info.name)) }
                    }
                }
            }
            @for (id, info) in &campaign.traits {
                @if let Some(text) = &info.description {
                    .description.hidden data-kind="trait" data-entity=(id) { (text) }
                }
            }
            ul.selection {
                @for (id, info) in &campaign.armorsets {
                    @let input_id = format!("armorset-{}", id);
                    li {
                        input id=(input_id) type="radio" name="armorset" value=(id);
                        label for=(input_id) { (capitalize(&info.title)) }
                    }
                }
            }
            ul.selection {
                @for (id, info) in &campaign.weaponsets {
                    @let input_id = format!("weaponset-{}", id);
                    li {
                        input id=(input_id) type="radio" name="weaponset" value=(id);
                        label for=(input_id) { (capitalize(&info.title)) }
                    }
                }
            }
            ul.selection {
                @for (id, info) in &campaign.locations {
                    @let input_id = format!("location-{}", id);
                    li {
                        input id=(input_id) type="radio" name="location" value=(id);
                        label for=(input_id) { (capitalize(&info.title)) }
                    }
                }
            }
            @for (id, info) in &campaign.locations {
                @if let Some(text) = &info.description {
                    .description.hidden data-kind="location" data-entity=(id) { (text) }
                }
            }
        }
    }
}

pub fn signup() -> Markup {
    html! {
        form.auth method="post" action="/signup" {
            fieldset {
                (field("text", "nick", "Ник"));
                (field("email", "email", "Email"));
                (field("password", "password", "Пароль"));
                (field("password", "password_confirm", "Повтор пароля"));
            }
            button type="submit" { "Регистрация" }
        }
    }
}

pub fn login() -> Markup {
    html! {
        form.auth method="post" action="/login" {
            fieldset {
                (field("text", "name", "Ник или email"));
                (field("password", "password", "Пароль"));
            }
            button type="submit" { "Вход" }
        }
    }
}
