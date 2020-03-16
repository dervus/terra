use maud::{html, Markup};
use crate::db::CharacterInfo;
use crate::system::{GenderFilter, Campaign};
use crate::util::capitalize;

pub fn campaign_page(campaign: &Campaign) -> Markup {
    html! {
        article {
            (maud::PreEscaped(&campaign.index_page))
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

pub fn character_index(campaign: &Campaign, characters: &[CharacterInfo]) -> Markup {
    html! {
        ul {
            @for pc in characters {
                li {
                    a href="#" {
                        (pc.name);
                        @if let Some(name_extra) = &pc.name_extra { " "; (name_extra) }
                    }
                    " – ";
                    (campaign.system.races.get(&pc.race).map(|info| info.name.get(pc.gender)).unwrap_or(&pc.race));
                    " ";
                    (campaign.system.classes.get(&pc.class).map(|info| info.name.get(pc.gender)).unwrap_or(&pc.class));
                    @if let Some(role) = &pc.role {
                        " (";
                        (campaign.blocks.iter().find(|b| b.roles.contains(role)).map(|b| b.name.clone()).unwrap_or("block".to_owned()));
                        " - ";
                        (campaign.roles.get(role).map(|info| info.name.clone()).unwrap_or(role.to_owned()));
                        ")";
                    }
                    @if pc.wants_loadup { " (Хочет загруз)" }
                    @if pc.hidden { " (Скрытый)" }
                }
            }
        }
    }
}

pub fn character_form(campaign: &Campaign, selected_role: Option<&str>) -> Markup {
    let maybe_role_with_id = selected_role.and_then(|id| campaign.roles.get(id).map(|role| (id, role)));
    let maybe_role = maybe_role_with_id.map(|pair| pair.1);

    let action_route = if let Some((role_id, _)) = maybe_role_with_id {
        format!("/characters/new/{}", role_id)
    } else {
        "/characters/new".to_owned()
    };

    let male_only = maybe_role.map(|r| r.gender == GenderFilter::MaleOnly).unwrap_or(false);
    let female_only = maybe_role.map(|r| r.gender == GenderFilter::FemaleOnly).unwrap_or(false);

    let mut races: Vec<_> = campaign.system.races.iter()
        .filter(|(race_id, _)| maybe_role.map(|role| role.races.matches(race_id)).unwrap_or(true))
        .collect();
    races.sort_by_key(|(_, info)| info.game_id);

    let mut classes: Vec<_> = campaign.system.classes.iter()
        .filter(|(class_id, _)| maybe_role.map(|role| role.classes.matches(class_id)).unwrap_or(true))
        .collect();
    classes.sort_by_key(|(_, info)| info.game_id);

    let mut armorsets: Vec<_> = campaign.system.armorsets.iter()
        .filter(|(armorset_id, _)| maybe_role.map(|role| role.armorsets.matches(armorset_id)).unwrap_or(true))
        .collect();
    armorsets.sort_by_key(|(_, info)| &info.name);

    let mut weaponsets: Vec<_> = campaign.system.weaponsets.iter()
        .filter(|(weaponset_id, _)| maybe_role.map(|role| role.weaponsets.matches(weaponset_id)).unwrap_or(true))
        .collect();
    weaponsets.sort_by_key(|(_, info)| &info.name);

    let mut traits: Vec<_> = campaign.system.traits.iter()
        .filter(|(trait_id, _)| maybe_role.map(|role| role.traits.matches(trait_id)).unwrap_or(true))
        .collect();
    traits.sort_by_key(|(_, info)| (-info.cost, &info.name));

    let mut locations: Vec<_> = campaign.system.locations.iter()
        .filter(|(location_id, _)| maybe_role.map(|role| role.locations.matches(location_id)).unwrap_or(true))
        .collect();
    locations.sort_by_key(|(_, info)| &info.name);

    fn descriptions<'a, I: 'a, T: 'a>(kind: &str, entities: I) -> Markup
    where
        I: std::iter::Iterator<Item = &'a (&'a String, &'a T)>,
        T: crate::system::Entity,
    {
        html! {
            @for (id, entity) in entities {
                 @if entity.description().is_some() || entity.preview().is_some() {
                    .entity-info.hidden data-kind=(kind) data-entity=(id) {
                        .name { (capitalize(entity.name())) }
                        @if let Some(text) = entity.description() {
                            .description { (text) }
                        }
                        @if let Some(path) = entity.preview() {
                            img.preview src=(format!("/assets/{}", path));
                        }
                    }
                }
            }
        }
    }

    html! {
        h1 { "Новый персонаж" }
        form#character-form
            method="post"
            action=(action_route)
            data-max-traits=(campaign.manifest.max_traits)
            data-max-traits-cost=(campaign.manifest.max_traits_cost)
        {
            .form-main {
                h2.form-header { "Роль" }
                .form-inputs {
                    @if let Some((role_id, role)) = maybe_role_with_id {
                        .role-name { (role.name) }
                        @if let Some(description) = &role.description {
                            .role-description { (description) }
                        }
                    } @else {
                        .role-name { "<Без роли>" }
                    }
                    .role-switch {
                        a href="/roles" { "Выбрать другую роль" }
                    }
                }
                aside.form-info {}
                h2.form-header { "Вид" }
                fieldset#race-section.form-inputs {
                    ul.selection {
                        @for (race_id, race) in &races {
                            @let input_id = format!("race-{}", race_id);
                            li {
                                input id=(input_id)
                                    type="radio"
                                    name="race"
                                    value=(race_id)
                                    required?
                                    checked?[races.len() == 1];

                                label for=(input_id) { (capitalize(race.name.male())) }
                            }
                        }
                    }
                    ul.selection {
                        li {
                            input id="gender-male"
                                type="radio"
                                name="gender"
                                value="male"
                                required?
                                checked?[male_only]
                                disabled?[female_only];

                            label for="gender-male" { "\u{2642}" }
                        }
                        li {
                            input id="gender-female"
                                type="radio"
                                name="gender"
                                value="female"
                                required?
                                checked?[female_only]
                                disabled?[male_only];

                            label for="gender-female" { "\u{2640}" }
                        }
                    }
                    @for (race_id, race) in &races {
                        ul.selection.hidden data-race=(race_id) {
                            @for (model_id, model) in &race.models {
                                @let input_id = format!("model-{}-{}", race_id, model_id);
                                li {
                                    input id=(input_id)
                                        type="radio"
                                        name="model"
                                        value=(model_id)
                                        required?
                                        checked?[race.models.len() == 1];

                                    label for=(input_id) { (model.name) }
                                }
                            }
                        }
                    }
                }
                aside.form-info {
                    .error { "Обязательный выбор" }
                    (descriptions("race", races.iter()));
                }
                h2.form-header { "Класс" }
                fieldset#class-section.form-inputs {
                    ul.selection {
                        @for (class_id, class) in &classes {
                            @let input_id = format!("class-{}", class_id);
                            li {
                                input id=(input_id)
                                    type="radio"
                                    name="class"
                                    value=(class_id)
                                    required?
                                    checked?[classes.len() == 1]
                                    data-race-filter=(class.races)
                                    data-gender-filter=(class.gender)
                                    data-armor-skill=(class.armor_skill_index())
                                    data-weapon-skills=(class.weapon_skills_string());

                                label for=(input_id) { (capitalize(class.name.male())) }
                            }
                        }
                    }
                }
                aside.form-info {
                    .error { "Обязательный выбор" }
                    (descriptions("class", classes.iter()));
                }
                h2.form-header.second-part { "Экипировка" }
                fieldset#armorset-section.form-inputs.second-part {
                    ul.selection {
                        @for (id, info) in &armorsets {
                            @let input_id = format!("armorset-{}", id);
                            li {
                                input id=(input_id)
                                    type="radio"
                                    name="armorset"
                                    value=(id)
                                    required?
                                    checked?[armorsets.len() == 1]
                                    data-race-filter=(info.races)
                                    data-gender-filter=(info.gender)
                                    data-class-filter=(info.classes)
                                    data-armor-skill=(info.armor_skill_index());

                                label for=(input_id) { (&info.name) }
                            }
                        }
                    }
                }
                aside.form-info.second-part {
                    .error { "Обязательный выбор" }
                    (descriptions("armorset", armorsets.iter()));
                }
                h2.form-header.second-part { "Оружее" }
                fieldset#armorset-section.form-inputs.second-part {
                    ul.selection {
                        @for (id, info) in &weaponsets {
                            @let input_id = format!("weaponset-{}", id);
                            li {
                                input id=(input_id)
                                    type="radio"
                                    name="weaponset"
                                    value=(id)
                                    required?
                                    checked?[weaponsets.len() == 1]
                                    data-race-filter=(info.races)
                                    data-gender-filter=(info.gender)
                                    data-class-filter=(info.classes)
                                    data-weapon-skills=(info.weapon_skills_string());

                                label for=(input_id) { (&info.name) }
                            }
                        }
                    }
                }
                aside.form-info.second-part {
                    .error { "Обязательный выбор" }
                    (descriptions("weaponset", weaponsets.iter()));
                }
                h2.form-header.second-part { "Особенности" }
                fieldset#traits-section.form-inputs.second-part {
                    ul.selection {
                        @for (id, info) in &traits {
                            @let input_id = format!("trait-{}", id);
                            li {
                                input id=(input_id)
                                    type="checkbox"
                                    name="traits"
                                    value=(id)
                                    data-cost=(info.cost)
                                    data-race-filter=(info.races)
                                    data-gender-filter=(info.gender)data-class-filter=(info.classes);

                                label for=(input_id) {
                                    span.cost.positive[info.cost > 0].negative[info.cost < 0].free[info.cost == 0] {
                                        (info.cost)
                                    }
                                    (&info.name)
                                }
                            }
                        }
                    }
                }
                aside.form-info.second-part {
                    (descriptions("trait", traits.iter()));
                }
                h2.form-header.second-part { "Локация" }
                fieldset#location-section.form-inputs.second-part {
                    ul.selection {
                        @for (id, info) in &locations {
                            @let input_id = format!("location-{}", id);
                            li {
                                input id=(input_id)
                                    type="radio"
                                    name="location"
                                    value=(id)
                                    required?
                                    checked?[locations.len() == 1]
                                    data-race-filter=(info.races)
                                    data-gender-filter=(info.gender)
                                    data-class-filter=(info.classes);

                                label for=(input_id) { (&info.name) }
                            }
                        }
                    }
                }
                aside.form-info.second-part {
                    .error { "Обязательный выбор" }
                    (descriptions("location", locations.iter()));
                }
                h2.form-header.second-part { "Информация" }
                fieldset#text-section.form-inputs.second-part {
                    div {
                        input id="name" type="text" name="name" minlength="2" maxlength="12" required? placeholder="Имя";
                        input id="name-extra" type="text" name="name_extra" maxlength="20" placeholder="Фамилия или другое";
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
                        input id="wants-loadup" type="checkbox" name="wants_loadup";
                        label for="wants-loadup" { "Хочу индивидуальный загруз" }
                    }
                    div {
                        input id="hidden" type="checkbox" name="hidden";
                        label for="hidden" { "Скрыть из общего списка" }
                    }
                }
                aside.form-info.second-part {
                    .error { "Обязательный выбор" }
                }
            }
            .form-controls {
                button type="submit" { "Готово" }
            }
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
