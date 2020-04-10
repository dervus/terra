use std::num::NonZeroU32;
use std::collections::{HashSet, HashMap};
use serde::Deserialize;
use crate::tags::Constraint;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Info {
    pub name: String,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub preview: Option<String>,
    #[serde(default)] pub requires: Option<Constraint>,
    #[serde(default)] pub provides: HashSet<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mods {
    #[serde(default)] pub spells_banned: HashSet<u32>,
    #[serde(default)] pub spells: HashSet<u32>,
    #[serde(default)] pub skills: HashMap<u32, i16>,
    #[serde(default)] pub items: HashMap<u32, i16>,
    #[serde(default)] pub money: i32,
}

impl Mods {
    pub fn join<'a, I>(mods: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<Self>
    {
        let mut out = Self::default();
        for x in mods { out.merge_in(x) }
        out
    }

    pub fn merge_in(&mut self, other: &Self) {
        fn sum(into: &mut HashMap<u32, i16>, from: &HashMap<u32, i16>) {
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
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Race {
    #[serde(flatten)] pub info: Info,
    #[serde(default)] pub name_female: Option<String>,
    pub game_id: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Class {
    #[serde(flatten)] pub info: Info,
    #[serde(default)] pub name_female: Option<String>,
    pub game_id: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Armor {
    #[serde(flatten)] pub info: Info,
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
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Weapon {
    #[serde(flatten)] pub info: Info,
    #[serde(default)] pub mainhand: Option<NonZeroU32>,
    #[serde(default)] pub offhand: Option<NonZeroU32>,
    #[serde(default)] pub ranged: Option<NonZeroU32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trait {
    #[serde(flatten)] pub info: Info,
    #[serde(default)] pub cost: i32,
    #[serde(flatten)] pub mods: Mods,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    #[serde(flatten)] pub info: Info,
    pub map: u32,
    pub zone: u32,
    pub position: (f32, f32, f32),
    pub orientation: f32,
    #[serde(flatten)] pub mods: Mods,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct System {
    #[serde(default)] pub race: HashMap<String, Race>,
    #[serde(default)] pub class: HashMap<String, Class>,
    #[serde(default)] pub armor: HashMap<String, Armor>,
    #[serde(default)] pub weapon: HashMap<String, Weapon>,
    #[serde(default, rename = "trait")] pub traits: HashMap<String, Trait>,
    #[serde(default)] pub location: HashMap<String, Location>,
}

impl System {
    pub fn merge_in(&mut self, other: &System) {
        fn walk<T: Clone>(into: &mut HashMap<String, T>, from: &HashMap<String, T>) {
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

    pub fn view<'a>(&'a self) -> SystemView<'a> {
        SystemView::new(self)
    }
}

pub struct SystemView<'a> {
    pub race: Vec<(&'a String, &'a Race)>,
    pub class: Vec<(&'a String, &'a Class)>,
    pub armor: Vec<(&'a String, &'a Armor)>,
    pub weapon: Vec<(&'a String, &'a Weapon)>,
    pub traits: Vec<(&'a String, &'a Trait)>,
    pub location: Vec<(&'a String, &'a Location)>,
}

impl<'a> SystemView<'a> {
    pub fn new(system: &'a System) -> Self {
        let mut race = system.race.iter().collect();
        let mut class = system.class.iter().collect();
        let mut armor = system.armor.iter().collect();
        let mut weapon = system.weapon.iter().collect();
        let mut traits = system.traits.iter().collect();
        let mut location = system.location.iter().collect();

        race.sort_by_key(|(_, e)| e.game_id);
        class.sort_by_key(|(_, e)| e.game_id);
        armor.sort_by_key(|(_, e)| &e.info.name);
        weapon.sort_by_key(|(_, e)| &e.info.name);
        traits.sort_by_key(|(_, e)| (-e.cost, &e.info.name));
        location.sort_by_key(|(_, e)| &e.info.name);

        Self { race, class, armor, weapon, traits, location }
    }
}


pub trait Entity {
    fn kind(&self) -> &'static str;
    fn info(&self) -> &Info;

    fn name(&self) -> &str {
        &self.info().name
    }
    fn description(&self) -> Option<&str> {
        self.info().description.as_ref()
    }
    fn preview(&self) -> Option<&str> {
        self.info().preview.as_ref()
    }
    fn requires(&self) -> Option<String> {
        self.info().requires.map(|c| c.into_string())
    }
    fn provides(&self) -> Option<String> {
        if !self.info().provides.is_empty() {
            Some(self.info().provides.iter().collect::<Vec<_>>().join(" "))
        } else {
            None
        }
    }
}

impl Entity for Race {
    fn kind(&self) -> &'static str { "race" }
    fn info(&self) -> &Info { &self.info }
}
impl Entity for Class {
    fn kind(&self) -> &'static str { "class" }
    fn info(&self) -> &Info { &self.info }
}
impl Entity for Armor {
    fn kind(&self) -> &'static str { "armor" }
    fn info(&self) -> &Info { &self.info }
}
impl Entity for Weapon {
    fn kind(&self) -> &'static str { "weapon" }
    fn info(&self) -> &Info { &self.info }
}
impl Entity for Trait {
    fn kind(&self) -> &'static str { "trait" }
    fn info(&self) -> &Info { &self.info }
}
impl Entity for Location {
    fn kind(&self) -> &'static str { "location" }
    fn info(&self) -> &Info { &self.info }
}
