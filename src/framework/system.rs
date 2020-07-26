use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    num::NonZeroU32,
};
use serde::{Deserialize, Serialize};
use super::tags::{Condition, Tags};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub name: String,
    #[serde(default)] pub name_female: Option<String>,
    #[serde(default)] pub info: Option<String>,
    #[serde(default)] pub preview: Option<String>,
    #[serde(default)] pub requires: Option<Condition>,
    #[serde(default)] pub provides: Tags,
    #[serde(default)] pub order: i32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mods {
    #[serde(default)] pub spells_banned: HashSet<NonZeroU32>,
    #[serde(default)] pub spells: HashSet<NonZeroU32>,
    #[serde(default)] pub skills: HashMap<NonZeroU32, i32>,
    #[serde(default)] pub items: HashMap<NonZeroU32, i32>,
    #[serde(default)] pub money: i32,
    #[serde(default)] pub level: i32,
}

impl Mods {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge_in(&mut self, other: &Self) {
        fn sum(into: &mut HashMap<NonZeroU32, i32>, from: &HashMap<NonZeroU32, i32>) {
            for (item, count) in from {
                if let Some(existing) = into.get_mut(item) {
                    *existing += count;
                } else {
                    into.insert(*item, *count);
                }
            }
        }

        self.spells_banned.extend(&other.spells_banned);
        self.spells.extend(&other.spells);
        sum(&mut self.skills, &other.skills);
        sum(&mut self.items, &other.items);
        self.money += other.money;
        self.level += other.level;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Race {
    #[serde(flatten)] pub meta: Metadata,
    pub game_id: u8,
    #[serde(flatten)] pub mods: Mods,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Class {
    #[serde(flatten)] pub meta: Metadata,
    pub game_id: u8,
    #[serde(flatten)] pub mods: Mods,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Armor {
    #[serde(flatten)] pub meta: Metadata,
    #[serde(default)] pub head: Option<NonZeroU32>,
    #[serde(default)] pub neck: Option<NonZeroU32>,
    #[serde(default)] pub shoulders: Option<NonZeroU32>,
    #[serde(default)] pub body: Option<NonZeroU32>,
    #[serde(default)] pub chest: Option<NonZeroU32>,
    #[serde(default)] pub waist: Option<NonZeroU32>,
    #[serde(default)] pub legs: Option<NonZeroU32>,
    #[serde(default)] pub feet: Option<NonZeroU32>,
    #[serde(default)] pub wrists: Option<NonZeroU32>,
    #[serde(default)] pub hands: Option<NonZeroU32>,
    #[serde(default)] pub fingers: Vec<NonZeroU32>,
    #[serde(default)] pub trinkets: Vec<NonZeroU32>,
    #[serde(default)] pub back: Option<NonZeroU32>,
    #[serde(default)] pub tabard: Option<NonZeroU32>,
    #[serde(default)] pub bags: Vec<NonZeroU32>,
    #[serde(flatten)] pub mods: Mods,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Weapon {
    #[serde(flatten)] pub meta: Metadata,
    #[serde(default)] pub mainhand: Option<NonZeroU32>,
    #[serde(default)] pub offhand: Option<NonZeroU32>,
    #[serde(default)] pub ranged: Option<NonZeroU32>,
    #[serde(flatten)] pub mods: Mods,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trait {
    #[serde(flatten)] pub meta: Metadata,
    #[serde(flatten)] pub mods: Mods,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    #[serde(flatten)] pub meta: Metadata,
    pub map: u32,
    pub zone: u32,
    pub position: (f32, f32, f32),
    pub orientation: f32,
    #[serde(flatten)] pub mods: Mods,
}

impl AsRef<Metadata> for Race {
    fn as_ref(&self) -> &Metadata {
        &self.meta
    }
}
impl AsRef<Metadata> for Class {
    fn as_ref(&self) -> &Metadata {
        &self.meta
    }
}
impl AsRef<Metadata> for Armor {
    fn as_ref(&self) -> &Metadata {
        &self.meta
    }
}
impl AsRef<Metadata> for Weapon {
    fn as_ref(&self) -> &Metadata {
        &self.meta
    }
}
impl AsRef<Metadata> for Trait {
    fn as_ref(&self) -> &Metadata {
        &self.meta
    }
}
impl AsRef<Metadata> for Location {
    fn as_ref(&self) -> &Metadata {
        &self.meta
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct System {
    #[serde(default)] pub race: HashMap<String, Race>,
    #[serde(default)] pub class: HashMap<String, Class>,
    #[serde(default)] pub armor: HashMap<String, Armor>,
    #[serde(default)] pub weapon: HashMap<String, Weapon>,
    #[serde(default, rename = "trait")] pub traits: HashMap<String, Trait>,
    #[serde(default)] pub location: HashMap<String, Location>,
}

impl System {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge_in(&mut self, other: &System) {
        fn walk<K, V>(into: &mut HashMap<K, V>, from: &HashMap<K, V>)
        where
            K: Clone + Hash + Eq,
            V: Clone,
        {
            for (key, value) in from {
                if !into.contains_key(key) {
                    into.insert(key.clone(), value.clone());
                }
            }
        }
        walk(&mut self.race, &other.race);
        walk(&mut self.class, &other.class);
        walk(&mut self.armor, &other.armor);
        walk(&mut self.weapon, &other.weapon);
        walk(&mut self.traits, &other.traits);
        walk(&mut self.location, &other.location);
    }

    pub fn view(&self) -> SystemView {
        SystemView::new(self)
    }

    pub fn info_iter(&self) -> impl Iterator<Item = &Metadata> {
        self.race
            .values()
            .map(AsRef::<Metadata>::as_ref)
            .chain(self.class.values().map(AsRef::<Metadata>::as_ref))
            .chain(self.armor.values().map(AsRef::<Metadata>::as_ref))
            .chain(self.weapon.values().map(AsRef::<Metadata>::as_ref))
            .chain(self.traits.values().map(AsRef::<Metadata>::as_ref))
            .chain(self.location.values().map(AsRef::<Metadata>::as_ref))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemView {
    pub race: HashMap<String, Metadata>,
    pub class: HashMap<String, Metadata>,
    pub armor: HashMap<String, Metadata>,
    pub weapon: HashMap<String, Metadata>,
    #[serde(rename = "trait")]
    pub traits: HashMap<String, Metadata>,
    pub location: HashMap<String, Metadata>,
}

impl SystemView {
    pub fn new(system: &System) -> Self {
        Self {
            race: system
                .race
                .iter()
                .map(|(id, data)| (id.clone(), data.meta.clone()))
                .collect(),
            class: system
                .class
                .iter()
                .map(|(id, data)| (id.clone(), data.meta.clone()))
                .collect(),
            armor: system
                .armor
                .iter()
                .map(|(id, data)| (id.clone(), data.meta.clone()))
                .collect(),
            weapon: system
                .weapon
                .iter()
                .map(|(id, data)| (id.clone(), data.meta.clone()))
                .collect(),
            traits: system
                .traits
                .iter()
                .map(|(id, data)| (id.clone(), data.meta.clone()))
                .collect(),
            location: system
                .location
                .iter()
                .map(|(id, data)| (id.clone(), data.meta.clone()))
                .collect(),
        }
    }
}
