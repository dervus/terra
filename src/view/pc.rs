use std::collections::HashSet;
use maud::{html, Markup};
use crate::framework::system::{Info, Trait};
use crate::framework::campaign::Campaign;
use crate::util;

// pub fn character_index(characters: &[CharacterInfo]) -> Markup {
//     html! {
//         ul {
//             @for pc in characters {
//                 li {
//                     a href="#" {
//                         (pc.name);
//                         @if let Some(name_extra) = &pc.name_extra { " "; (name_extra) }
//                     }
//                 }
//             }
//         }
//     }
// }

#[derive(Default)]
pub struct CharacterForm {
    pub role: Option<String>,
    pub female: Option<bool>,
    pub race: Option<u8>,
    pub class: Option<u8>,
    pub armor: Option<String>,
    pub weapon: Option<String>,
    pub traits: HashSet<String>,
    pub location: Option<String>,
    pub name: Option<String>,
    pub name_extra: Option<String>,
    pub info_public: Option<String>,
    pub info_hidden: Option<String>,
    pub loadup: bool,
    pub hidden: bool,
}

pub fn form(campaign: &Campaign, data: &CharacterForm) -> super::Page {
    super::Page::titled("Редактирование персонажа")
        .stylesheet("/assets/character_form.css")
        .script("/assets/character_form.js")
        .content(character_form(campaign, data))
}

pub fn character_form(campaign: &Campaign, data: &CharacterForm) -> Markup {
    let sv = campaign.system.view();
    html! {
        h1 { "Новый персонаж" }
        form#character-form method="post" {
            .form-main {
                //(fieldset_role(campaign.roles.iter()));
                (fieldset_entity("Вид", "race", &sv.race, data.race.as_ref()));
                (fieldset_gender(data.female));
                (fieldset_entity("Класс", "class", &sv.class, data.class.as_ref()));
                (fieldset_entity("Экипировка", "armor", &sv.armor, data.armor.as_ref()));
                (fieldset_entity("Оружее", "weapon", &sv.weapon, data.weapon.as_ref()));
                (fieldset_traits("Особенности", &sv.traits, &data.traits));
                (fieldset_entity("Локация", "location", &sv.location, data.location.as_ref()));
                
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
                            textarea name="info_public" {}
                        }
                        div {
                            "Пожелания, заметки и комментарии для мастеров";
                            br;
                            "Написаное здесь будет видно только мастерской группе, даже если персонаж не скрыт из общего списка";
                            textarea name="info_hidden" {}
                        }
                        div {
                            input id="loadup" type="checkbox" name="loadup";
                            label for="loadup" { "Хочу индивидуальный загруз" }
                        }
                        div {
                            input id="private" type="checkbox" name="private";
                            label for="private" { "Скрыть из общего списка" }
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

fn fieldset_gender(value: Option<bool>) -> Markup {
    html! {
        //fieldset {
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
                            selected?[value.map(|v| v == false).unwrap_or(false)]
                            data-provides="gender:male";

                        label for="gender-male" { "\u{2642}" }
                    }
                    li {
                        input id="gender-female"
                            type="radio"
                            name="gender"
                            value="female"
                            required?
                            selected?[value.map(|v| v == true).unwrap_or(false)]
                            data-provides="gender:female";

                        label for="gender-female" { "\u{2640}" }
                    }
                }
            }
            .form-info {}
        //}
    }
}

fn fieldset_entity<K, V>(title: &str, kind: &str, data: &[(K, V)], value: Option<K>) -> Markup
where
    K: std::string::ToString,
    V: AsRef<Info>,
{
    let value = value.map(|v| v.to_string());
    html! {
        //fieldset {
            .form-header {
                @if let Some(s) = title.into() {
                    h2 { (title) }
                }
            }
            .form-inputs {
                ul.selection {
                    @for (id, info) in data {
                        @let id = id.to_string();
                        @let info = info.as_ref();
                        @let input_id = format!("{}-{}", kind, id);
                        //@let selected = value.map(|v| &v == &id).unwrap_or(false);
                        @let selected = false;
                        li {
                            input id=(input_id)
                                type="radio"
                                name=(kind)
                                value=(id)
                                required?
                                selected?[selected]
                                data-preview=(info.preview.as_deref().unwrap_or(""))
                                data-info=(info.info.as_deref().unwrap_or(""))
                                data-requires=(info.make_requires_string())
                                data-provides=(info.make_provides_string());

                            label for=(input_id) { (util::capitalize(&info.name)) }
                        }
                    }
                }
            }
            .form-info {}
        //}
    }
}

fn fieldset_traits(title: &str, data: &[(&String, &Trait)], value: &HashSet<String>) -> Markup {
    html! {
        //fieldset {
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
                                data-preview=(trait_def.info.preview.as_deref().unwrap_or(""))
                                data-info=(trait_def.info.info.as_deref().unwrap_or(""))
                                data-requires=(trait_def.info.make_requires_string())
                                data-provides=(trait_def.info.make_provides_string());

                            label for=(input_id) { (util::capitalize(&trait_def.info.name)) }
                        }
                    }
                }
            }
            .form-info {}
        //}
    }
}
