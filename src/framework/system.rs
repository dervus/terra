use std::num::NonZeroU32;
use std::hash::Hash;
use std::collections::{HashSet, HashMap};
use serde::Deserialize;
use super::tags::Condition;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Info {
    pub name: String,
    #[serde(default)] pub info: Option<String>,
    #[serde(default)] pub preview: Option<String>,
    #[serde(default)] pub requires: Option<Condition>,
    #[serde(default)] pub provides: HashSet<String>,
}

impl Info {
    pub fn make_requires_string(&self) -> String {
        self.requires.as_ref().map(|c| c.to_string()).unwrap_or_else(String::new)
    }

    pub fn make_provides_string(&self) -> String {
        self.provides.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ")
    }
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
        for x in mods { out.merge_in(x.as_ref()) }
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
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Class {
    #[serde(flatten)] pub info: Info,
    #[serde(default)] pub name_female: Option<String>,
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

impl AsRef<Info> for Race {
    fn as_ref(&self) -> &Info { &self.info }
}
impl AsRef<Info> for Class {
    fn as_ref(&self) -> &Info { &self.info }
}
impl AsRef<Info> for Armor {
    fn as_ref(&self) -> &Info { &self.info }
}
impl AsRef<Info> for Weapon {
    fn as_ref(&self) -> &Info { &self.info }
}
impl AsRef<Info> for Trait {
    fn as_ref(&self) -> &Info { &self.info }
}
impl AsRef<Info> for Location {
    fn as_ref(&self) -> &Info { &self.info }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct System {
    #[serde(default)] pub race: HashMap<u8, Race>,
    #[serde(default)] pub class: HashMap<u8, Class>,
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
            V: Clone
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

    pub fn view<'a>(&'a self) -> SystemView<'a> {
        SystemView::new(self)
    }

    pub fn info_iter(&self) -> impl Iterator<Item = &Info> {
        self.race.values().map(AsRef::<Info>::as_ref)
            .chain(self.class.values().map(AsRef::<Info>::as_ref))
            .chain(self.armor.values().map(AsRef::<Info>::as_ref))
            .chain(self.weapon.values().map(AsRef::<Info>::as_ref))
            .chain(self.traits.values().map(AsRef::<Info>::as_ref))
            .chain(self.location.values().map(AsRef::<Info>::as_ref))
    }
}

pub struct SystemView<'a> {
    pub race: Vec<(&'a u8, &'a Race)>,
    pub class: Vec<(&'a u8, &'a Class)>,
    pub armor: Vec<(&'a String, &'a Armor)>,
    pub weapon: Vec<(&'a String, &'a Weapon)>,
    pub traits: Vec<(&'a String, &'a Trait)>,
    pub location: Vec<(&'a String, &'a Location)>,
}

impl<'a> SystemView<'a> {
    pub fn new(system: &'a System) -> Self {
        let mut race: Vec<_> = system.race.iter().collect();
        let mut class: Vec<_> = system.class.iter().collect();
        let mut armor: Vec<_> = system.armor.iter().collect();
        let mut weapon: Vec<_> = system.weapon.iter().collect();
        let mut traits: Vec<_> = system.traits.iter().collect();
        let mut location: Vec<_> = system.location.iter().collect();

        race.sort_by_key(|(id, _)| id.clone());
        class.sort_by_key(|(id, _)| id.clone());
        armor.sort_by_key(|(_, e)| &e.info.name);
        weapon.sort_by_key(|(_, e)| &e.info.name);
        traits.sort_by_key(|(_, e)| (-e.cost, &e.info.name));
        location.sort_by_key(|(_, e)| &e.info.name);

        Self { race, class, armor, weapon, traits, location }
    }
}
