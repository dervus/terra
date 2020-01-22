use maud::{html, Markup};
use crate::system::Campaign;
use crate::util::capitalize;

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
                a href="/characters/new" { "Новый персонаж" }
            }
        }
    }
}

pub fn character_form(campaign: &Campaign, selected_role: Option<&str>) -> Markup {
    let mut traits: Vec<_> = campaign.system.traits.iter().collect();
    traits.sort_by_key(|(_, info)| (-info.cost, &info.name));

    let preview_url = |template: &str| -> String {
        template
            .replace("{system}", "/system/assets")
            .replace("{campaign}", &format!("/campaigns/{}/assets", &campaign.id))
    };

    fn descriptions<T, F>(kind: &str, entities: &crate::system::EntityMap<T>, preview_fn: F) -> Markup
    where T: crate::system::Entity,
          F: Fn(&str) -> String
    {
        html! {
            @for (id, entity) in entities {
                 @if entity.description().is_some() || entity.preview().is_some() {
                    .entity-info.hidden data-kind=(kind) data-entity=(id) {
                        .name { (capitalize(entity.name())) }
                        @if let Some(text) = entity.description() {
                            .description { (text) }
                        }
                        @if let Some(template) = entity.preview() {
                            img.preview src=(preview_fn(template));
                        }
                    }
                }
            }
        }
    }

    html! {
        h1 { "Новый персонаж" }
        form {
            #character-form {
                h2.form-header { "Роль" }
                .form-inputs {
                    @if let Some(role_id) = selected_role {
                        @if let Some(role) = campaign.roles.get(role_id) {
                            .role-name { (role.name) }
                            @if let Some(description) = &role.description {
                                .role-description { (description) }
                            }
                        } @else {
                            .role-name { (role_id) }
                        }
                    } @else {
                        .role-name { "<Без роли>" }
                    }
                    .role-switch {
                        a href="#" { "Выбрать другую роль" }
                    }
                }
                aside.form-info {}
                h2.form-header { "Вид" }
                fieldset#race-section.form-inputs {
                    ul.selection {
                        @for (race_id, race) in &campaign.system.races {
                            @let input_id = format!("race-{}", race_id);
                            li {
                                input id=(input_id) type="radio" name="race" value=(race_id);
                                label for=(input_id) { (capitalize(race.name.male())) }
                            }
                        }
                    }
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
                    @for (race_id, race) in &campaign.system.races {
                        ul.selection.hidden data-race=(race_id) {
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
                aside.form-info {
                    (descriptions("race", &campaign.system.races, preview_url));
                }
                h2.form-header { "Класс" }
                fieldset#class-section.form-inputs {
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
                aside.form-info {
                    (descriptions("class", &campaign.system.classes, preview_url));
                }
                h2.form-header { "Экипировка" }
                fieldset#armorset-section.form-inputs {
                    ul.selection {
                        @for (id, info) in &campaign.system.armorsets {
                            @let input_id = format!("armorset-{}", id);
                            li {
                                input id=(input_id) type="radio" name="armorset" value=(id) data-race-filter=(info.races) data-class-filter=(info.classes);
                                label for=(input_id) { (capitalize(&info.name)) }
                            }
                        }
                    }
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
                aside.form-info {
                    (descriptions("armorset", &campaign.system.armorsets, preview_url));
                    (descriptions("weaponset", &campaign.system.weaponsets, preview_url));
                }
                h2.form-header { "Особенности" }
                fieldset#traits-section.form-inputs {
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
                }
                aside.form-info {
                    (descriptions("trait", &campaign.system.traits, preview_url));
                }
                h2.form-header { "Локация" }
                fieldset#location-section.form-inputs {
                    ul.selection {
                        @for (id, info) in &campaign.system.locations {
                            @let input_id = format!("location-{}", id);
                            li {
                                input id=(input_id) type="radio" name="location" value=(id) data-race-filter=(info.races) data-class-filter=(info.classes);
                                label for=(input_id) { (capitalize(&info.name)) }
                            }
                        }
                    }
                }
                aside.form-info {
                    (descriptions("location", &campaign.system.locations, preview_url));
                }
                h2.form-header { "Информация" }
                fieldset#text-section.form-inputs {
                    div {
                        input id="primary-name" type="text" name="primary_name" placeholder="Имя";
                        input id="secondary-name" type="text" name="secondary_name";
                    }
                    div {
                        "Общее описание персонажа и его место в мире";
                        textarea name="description" {}
                    }
                    div {
                        "Пожелания, заметки и комментарии для мастеров";
                        br;
                        "Написаное здесь будет видно только мастерской группе, даже если персонаж не скрыт из общего списка";
                        textarea name="comment" {}
                    }
                    div {
                        input id="request-loadup" type="checkbox" name="request_loadup";
                        label for="request-loadup" { "Хочу индивидуальный загруз" }
                    }
                    div {
                        input id="hidden" type="checkbox" name="hidden";
                        label for="hidden" { "Скрыть из общего списка" }
                    }
                }
                aside.form-info {}
            }
            .form-buttons {
                button type="submit" { "Готово" }
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
        form.auth method="post" action="/login" {
            fieldset {
                (field("text", "name", "Ник или email"));
                (field("password", "password", "Пароль"));
            }
            button type="submit" { "Вход" }
        }
    }
}
