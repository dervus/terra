use std::collections::HashSet;
use maud::{html, Markup};
use crate::db::{Character, CharacterInfo, Gender};
use crate::framework::system::{Info, Trait};
use crate::framework::campaign::Campaign;
use crate::util;

pub fn character_index(characters: &[CharacterInfo]) -> Markup {
    html! {
        ul {
            @for pc in characters {
                li {
                    a href="#" {
                        (pc.name);
                        @if let Some(name_extra) = &pc.name_extra { " "; (name_extra) }
                    }
                }
            }
        }
    }
}

#[derive(Default)]
pub struct CharacterForm {
    pub role: Option<String>,
    pub race: Option<String>,
    pub gender: Option<Gender>,
    pub class: Option<String>,
    pub armor: Option<String>,
    pub weapon: Option<String>,
    pub traits: HashSet<String>,
    pub location: Option<String>,
    pub name: Option<String>,
    pub name_extra: Option<String>,
    pub description: Option<String>,
    pub comment: Option<String>,
    pub loadup: bool,
    pub hidden: bool,
}

fn character_form(campaign: &Campaign, data: &CharacterForm) -> Markup {
    let sv = campaign.system.view();
    html! {
        h1 { "Новый персонаж" }
        form#character-form
            method="post"
            data-trait-limit=(campaign.trait_limit)
            data-trait-balance=(campaign.trait_balance)
        {
            .form-main {
                //(fieldset_role(campaign.roles.iter()));
                (fieldset_entity("Вид", "race", &sv.race, data.race.as_deref()));
                (fieldset_gender(data.gender));
                (fieldset_entity("Класс", "class", &sv.class, data.class.as_deref()));
                (fieldset_entity("Экипировка", "armor", &sv.armor, data.armor.as_deref()));
                (fieldset_entity("Оружее", "weapon", &sv.weapon, data.weapon.as_deref()));
                (fieldset_traits("Особенности", &sv.traits, &data.traits));
                (fieldset_entity("Локация", "location", &sv.location, data.location.as_deref()));
                
                fieldset {
                    .form-header {
                        h2 { "Информация" }
                    }
                    .form-inputs {
                        div {
                            input id="name" type="text" name="name" minlength="2" maxlength="12" required? placeholder="Имя";
                            input id="name_extra" type="text" name="name_extra" maxlength="20" placeholder="Фамилия или другое";
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
                            input id="loadup" type="checkbox" name="loadup";
                            label for="loadup" { "Хочу индивидуальный загруз" }
                        }
                        div {
                            input id="hidden" type="checkbox" name="hidden";
                            label for="hidden" { "Скрыть из общего списка" }
                        }
                    }
                    .form-info {}
                }
            }
            .form-controls {
                button type="submit" { "Готово" }
            }
        }
    }
}

// fn fieldset_role(data: &HashMap<String, Role>, value: Option<&str>) -> Markup {
//     html! {
//         fieldset {
//             .form-header {
//                 h2 { "Роль" }
//             }
//             .form-inputs {
//                 select name="role" required? {
//                     @for (id, role) in data {
//                         @let selected = value.map(|v| v == id).unwrap_or(false);
//                         option value=(id) selected?[selected] { (role.name) }
//                     }
//                 }
//             }
//             .form-info {}
//         }
//     }
// }

fn fieldset_gender(value: Option<Gender>) -> Markup {
    html! {
        fieldset {
            .form-header {
                h2 { "Пол" }
            }
            .form-inputs {
                ul.selection {
                    li {
                        input id="gender-male"
                            type="radio"
                            name="gender"
                            value="male"
                            required?
                            selected?[value.map(|v| v == Gender::Male).unwrap_or(false)]
                            data-provides="gender:male";

                        label for="gender-male" { "\u{2642}" }
                    }
                    li {
                        input id="gender-female"
                            type="radio"
                            name="gender"
                            value="female"
                            required?
                            selected?[value.map(|v| v == Gender::Female).unwrap_or(false)]
                            data-provides="gender:female";

                        label for="gender-female" { "\u{2640}" }
                    }
                }
            }
            .form-info {}
        }
    }
}

fn fieldset_entity<T: AsRef<Info>>(title: &str, kind: &str, data: &[(&String, T)], value: Option<&str>) -> Markup {
    html! {
        fieldset {
            .form-header {
                @if let Some(s) = title.into() {
                    h2 { (title) }
                }
            }
            .form-inputs {
                ul.selection {
                    @for (id, as_info) in data {
                        @let info = as_info.as_ref();
                        @let input_id = format!("{}-{}", kind, id);
                        li {
                            input id=(input_id)
                                type="radio"
                                name=(kind)
                                value=(id)
                                required?
                                selected?[value.map(|v| v == id.as_str()).unwrap_or(false)]
                                data-requires=(info.make_requires_string())
                                data-provides=(info.make_requires_string());

                            label for=(input_id) { (util::capitalize(&info.name)) }
                        }
                    }
                }
            }
            .form-info {
                @for (id, as_info) in data {
                    @let info = as_info.as_ref();
                    @let input_id = format!("{}-{}", kind, id);
                    .entity-info.hidden data-input=(input_id) {
                        .name { (util::capitalize(info.name)) }
                        @if let Some(text) = info.description {
                            .description { (text) }
                        }
                        @if let Some(path) = info.preview {
                            @let url = format!("/assets/{}", path);
                            @if path.ends_with(".webm") {
                                video muted? autoplay? loop? src=(url) { "[Предпросмотр]" }
                            } @else {
                                img.preview src=(url) alt="[Предпросмотр]";
                            }
                        }
                    }
                }
            }
        }
    }
}

fn fieldset_traits(title: &str, data: &[(&String, &Trait)], value: &HashSet<String>) -> Markup {
    html! {
        fieldset {
            .form-header {
                @if let Some(s) = title.into() {
                    h2 { (title) }
                }
            }
            .form-inputs {
                ul.selection {
                    @for (id, trait_def) in data {
                        @let input_id = format!("trait-{}", id);
                        li {
                            input id=(input_id)
                                type="checkbox"
                                name="trait"
                                value=(id)
                                selected?[value.contains(id.as_str())]
                                data-requires=(trait_def.info.make_requires_string())
                                data-provides=(trait_def.info.make_provides_string());

                            label for=(input_id) { (util::capitalize(&trait_def.info.name)) }
                        }
                    }
                }
            }
            .form-info {}
        }
    }
}
