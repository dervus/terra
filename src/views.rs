use maud::{html, Markup};
use crate::system::Campaign;
use crate::util::capitalize;
use crate::handlers;

pub fn campaign(campaign: &Campaign) -> Markup {
    html! {
        article {
            (maud::PreEscaped(&campaign.index_page))
        }
        ul.blocks {
            @for block in &campaign.blocks {
                li {
                    span { (block.name) }
                    ul {
                        @for role_id in &block.roles {
                            @if let Some(role) = campaign.roles.get(role_id) {
                                li {
                                    (&role.name)
                                }
                            } @else {
                                li { "[missing: " (role_id) "]" }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn characters() -> Markup {
    html! {
        ul {
            li {
                a href=(uri!(handlers::new_character: _)) { "Новый персонаж" }
            }
        }
    }
}

pub fn character_form(campaign: &Campaign, selected_role: Option<&str>) -> Markup {
    let mut traits: Vec<_> = campaign.system.traits.iter().collect();
    traits.sort_by_key(|(_, info)| (-info.cost, &info.name));

    html! {
        h1 { "Новый персонаж" }
        form {
            fieldset#role-section {
                ul {
                    @for block in &campaign.blocks {
                        li {
                            span { (block.name) }
                            ul {
                                @for role_id in &block.roles {
                                    @let input_id = format!("role-{}", role_id);
                                    @if let Some(role) = campaign.roles.get(role_id) {
                                        li {
                                            input id=(input_id) type="radio" name="role" value=(role_id) checked?[selected_role.map(|sel| sel == role_id).unwrap_or(false)];
                                            label for=(input_id) { (capitalize(&role.name)) }
                                        }
                                    } @else {
                                        li { "[missing: " (role_id) "]" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            fieldset#gender-section {
                ul.selection {
                    li {
                        input id="gender-male" type="radio" name="gender" value="male";
                        label for="gender-male" { "\u{2642}" }
                    }
                    li {
                        input id="gender-female" type="radio" name="gender" value="female";
                        label for="gender-female" { "\u{2640}" }
                    }
                }
            }
            fieldset#race-section {
                ul.selection {
                    @for (race_id, race) in &campaign.system.races {
                        @let input_id = format!("race-{}", race_id);
                        li {
                            input id=(input_id) type="radio" name="race" value=(race_id);
                            label for=(input_id) { (capitalize(race.name.male())) }
                        }
                    }
                }
            }
            fieldset#model-section {
                @for (race_id, race) in &campaign.system.races {
                    ul.selection data-race=(race_id) {
                        @for (model_id, model) in &race.models {
                            @let input_id = format!("model-{}-{}", race_id, model_id);
                            li {
                                input id=(input_id) type="radio" name="model" value=(model_id);
                                label for=(input_id) { (model.name) }
                            }
                        }
                    }
                }
            }
            fieldset#class-section {
                ul.selection {
                    @for (class_id, class) in &campaign.system.classes {
                        @let input_id = format!("class-{}", class_id);
                        li {
                            input id=(input_id) type="radio" name="class" value=(class_id) data-race-filter=(class.races);
                            label for=(input_id) { (capitalize(class.name.male())) }
                        }
                    }
                }
            }
            fieldset#traits-section {
                ul.selection {
                    @for (id, info) in &traits {
                        @let input_id = format!("trait-{}", id);
                        li {
                            input id=(input_id) type="checkbox" name="trait" value=(id) data-cost=(info.cost) data-race-filter=(info.races) data-class-filter=(info.classes);
                            label for=(input_id) {
                                span.cost.positive[info.cost > 0].negative[info.cost < 0].free[info.cost == 0] {
                                    (info.cost)
                                }
                                (capitalize(&info.name))
                            }
                        }
                    }
                }
                @for (id, info) in &campaign.system.traits {
                    @if let Some(text) = &info.description {
                        .description.hidden data-kind="trait" data-entity=(id) { (text) }
                    }
                }
            }
            fieldset#armorset-section {
                ul.selection {
                    @for (id, info) in &campaign.system.armorsets {
                        @let input_id = format!("armorset-{}", id);
                        li {
                            input id=(input_id) type="radio" name="armorset" value=(id) data-race-filter=(info.races) data-class-filter=(info.classes);
                            label for=(input_id) { (capitalize(&info.name)) }
                        }
                    }
                }
            }
            fieldset#weaponset-section {
                ul.selection {
                    @for (id, info) in &campaign.system.weaponsets {
                        @let input_id = format!("weaponset-{}", id);
                        li {
                            input id=(input_id) type="radio" name="weaponset" value=(id) data-race-filter=(info.races) data-class-filter=(info.classes);
                            label for=(input_id) { (capitalize(&info.name)) }
                        }
                    }
                }
            }
            fieldset#location-section {
                ul.selection {
                    @for (id, info) in &campaign.system.locations {
                        @let input_id = format!("location-{}", id);
                        li {
                            input id=(input_id) type="radio" name="location" value=(id) data-race-filter=(info.races) data-class-filter=(info.classes);
                            label for=(input_id) { (capitalize(&info.name)) }
                        }
                    }
                }
                @for (id, info) in &campaign.system.locations {
                    @if let Some(text) = &info.description {
                        .description.hidden data-kind="location" data-entity=(id) { (text) }
                    }
                }
            }
            fieldset#text-section {
                input id="main-name" type="text" name="main_name" placeholder="Имя";
                input id="rest-name" type="text" name="rest_name";
                
                textarea name="description" {}
                textarea name="comment" {}
                
                input id="request-loadup" type="checkbox" name="request_loadup";
                label for="request-loadup" { "Хочу индивидуальный загруз" }
                
                input id="hidden" type="checkbox" name="hidden";
                label for="hidden" { "Скрыть из общего списка" }
            }
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
        form.auth method="post" {
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
        form.auth method="post" action=(uri!(handlers::login_action)) {
            fieldset {
                (field("text", "name", "Ник или email"));
                (field("password", "password", "Пароль"));
            }
            button type="submit" { "Вход" }
        }
    }
}
