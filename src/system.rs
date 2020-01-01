use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs::{File, read_to_string};
use std::io::BufReader;

use anyhow::anyhow;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use serde_yaml::Value as YamlValue;
use comrak::{ComrakOptions, markdown_to_html};

pub type Result<T> = std::result::Result<T, anyhow::Error>;
pub type EntityId = String;
pub type EntityIds = HashSet<EntityId>;
pub type EntityMap<T> = HashMap<EntityId, T>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Filter {
    Pass,
    Allow(EntityIds),
    Deny(EntityIds),
}

impl Default for Filter {
    fn default() -> Self { Filter::Pass }
}

impl Filter {
    pub fn check(&self, id: &str) -> bool {
        match self {
            Filter::Pass => true,
            Filter::Allow(s) => s.contains(id),
            Filter::Deny(s) => !s.contains(id),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub enum Name {
//     Unisex(String),
//     Double(String, String),
// }
pub type Name = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmorSet {
    pub title: String,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub preview: Option<String>,
    #[serde(default)] pub races: Filter,
    #[serde(default)] pub gender: Option<Gender>,
    #[serde(default)] pub classes: Filter,
    #[serde(default)] pub head: Option<u32>,
    #[serde(default)] pub neck: Option<u32>,
    #[serde(default)] pub shoulders: Option<u32>,
    #[serde(default)] pub body: Option<u32>,
    #[serde(default)] pub chest: Option<u32>,
    #[serde(default)] pub waist: Option<u32>,
    #[serde(default)] pub legs: Option<u32>,
    #[serde(default)] pub feet: Option<u32>,
    #[serde(default)] pub wrists: Option<u32>,
    #[serde(default)] pub hands: Option<u32>,
    #[serde(default)] pub fingers: Vec<u32>,
    #[serde(default)] pub trinkets: Vec<u32>,
    #[serde(default)] pub back: Option<u32>,
    #[serde(default)] pub tabard: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponSet {
    pub title: String,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub preview: Option<String>,
    #[serde(default)] pub races: Filter,
    #[serde(default)] pub gender: Option<Gender>,
    #[serde(default)] pub classes: Filter,
    #[serde(default)] pub mainhand: Option<u32>,
    #[serde(default)] pub offhand: Option<u32>,
    #[serde(default)] pub ranged: Option<u32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtraMods {
    #[serde(default)] pub spells: HashSet<u32>,
    #[serde(default)] pub skills: HashMap<u32, i16>,
    #[serde(default)] pub factions: HashMap<u32, i16>,
    #[serde(default)] pub items: HashMap<u32, i16>,
    #[serde(default)] pub money: i32,
}

impl ExtraMods {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: &Self) {
        fn merge_amounts(into: &mut HashMap<u32, i16>, from: &HashMap<u32, i16>) {
            for (item, count) in from {
                if let Some(existing) = into.get_mut(item) {
                    *existing += count;
                } else {
                    into.insert(*item, *count);
                }
            }
        }
        
        self.spells.extend(&other.spells);
        merge_amounts(&mut self.skills, &other.skills);
        merge_amounts(&mut self.factions, &other.factions);
        merge_amounts(&mut self.items, &other.items);
        self.money += other.money;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Race {
    pub name: Name,
    pub game_id: u32,
    #[serde(default)] pub models: EntityMap<Model>,
}

fn default_model_scale() -> f32 { 1.0 }
fn default_model_speed() -> f32 { 1.0 }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub name: String,
    pub display_id: u32,
    pub gender: Gender,
    #[serde(default = "default_model_scale")] pub scale: f32,
    #[serde(default = "default_model_speed")] pub speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Class {
    pub name: Name,
    pub game_id: u32,
    #[serde(default)] pub races: Filter,
    #[serde(default)] pub gender: Option<Gender>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trait {
    pub cost: i32,
    pub name: Name,
    #[serde(default)] pub group: Option<String>,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub races: Filter,
    #[serde(default)] pub gender: Option<Gender>,
    #[serde(default)] pub classes: Filter,
    #[serde(default, flatten)] pub mods: ExtraMods,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub title: String,
    #[serde(default)] pub description: Option<String>,
    pub map: u32,
    pub zone: u32,
    pub position: (f32, f32, f32),
    pub orientation: f32,
    #[serde(default)] pub mods: ExtraMods,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleKind {
    Free,
    Normal,
    Special,
}

impl Default for RoleKind {
    fn default() -> Self { RoleKind::Normal }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Role {
    pub title: String,
    #[serde(default)] pub limit: Option<u32>,
    #[serde(default)] pub kind: RoleKind,
    #[serde(default)] pub gender: Option<Gender>,
    #[serde(default)] pub common: Option<ExtraMods>,
    #[serde(default)] pub races: EntityMap<Option<ExtraMods>>,
    #[serde(default)] pub classes: EntityMap<Option<ExtraMods>>,
    #[serde(default)] pub starters: Filter,
    #[serde(default)] pub traits: Filter,
    #[serde(default)] pub locations: Filter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub title: String,
    #[serde(default)] pub description: Option<String>,
    pub roles: Vec<EntityId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub title: String,
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Campaign {
    pub title: String,
    #[serde(default)] pub pages: Vec<Page>,
    pub max_traits: u32,
    pub max_traits_cost: i32,
    #[serde(default)] pub races: HashMap<EntityId, Race>,
    #[serde(default)] pub classes: HashMap<EntityId, Class>,
    #[serde(default)] pub armorsets: HashMap<EntityId, ArmorSet>,
    #[serde(default)] pub weaponsets: HashMap<EntityId, WeaponSet>,
    #[serde(default)] pub traits: HashMap<EntityId, Trait>,
    #[serde(default)] pub locations: HashMap<EntityId, Location>,
    pub blocks: Vec<Block>,
    pub roles: HashMap<EntityId, Role>,
}

#[derive(Debug)]
pub struct SystemLoader {
    system_path: PathBuf,
    races: EntityMap<Race>,
    classes: EntityMap<Class>,
    armorsets: EntityMap<ArmorSet>,
    weaponsets: EntityMap<WeaponSet>,
    traits: EntityMap<Trait>,
    locations: EntityMap<Location>,
}

impl SystemLoader {
    pub fn init<P: Into<PathBuf>>(system_path: P) -> Result<Self> {
        let system_path = system_path.into();
        let path = system_path.join("shared");

        let races = load_yaml(&path.join("races.yml"))?;
        let classes = load_yaml(&path.join("classes.yml"))?;
        let armorsets = load_yaml(&path.join("armorsets.yml"))?;
        let weaponsets = load_yaml(&path.join("weaponsets.yml"))?;
        let traits = load_yaml(&path.join("traits.yml"))?;
        let locations = load_yaml(&path.join("locations.yml"))?;

        Ok(Self { system_path, races, classes, armorsets, weaponsets, traits, locations })
    }

    pub fn load_campaign(&self, id: &str) -> Result<Campaign> {
        let campaign_path = self.system_path.join("campaigns").join(id);
        let campaign_file = File::open(campaign_path.join("manifest.yml"))?;
        let mut campaign: Campaign = serde_yaml::from_reader(BufReader::new(campaign_file))?;

        fn merge<T: Clone>(into: &mut EntityMap<T>, from: &EntityMap<T>) {
            for (key, value) in from {
                if !into.contains_key(key) {
                    into.insert(key.clone(), value.clone());
                }
            }
        }
        merge(&mut campaign.races, &self.races);
        merge(&mut campaign.classes, &self.classes);
        merge(&mut campaign.armorsets, &self.armorsets);
        merge(&mut campaign.weaponsets, &self.weaponsets);
        merge(&mut campaign.traits, &self.traits);
        merge(&mut campaign.locations, &self.locations);

        Ok(campaign)
    }
}

fn load_yaml<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    if path.exists() {
        let file = File::open(path)?;
        let yaml: T = serde_yaml::from_reader(BufReader::new(file))?;
        Ok(yaml)
    } else {
        Ok(Default::default())
    }
}

fn prepare_campaign_description(input: &str) -> String {
    let options = ComrakOptions {
        smart: true,
        ext_strikethrough: true,
        ext_table: true,
        ext_autolink: true,
        ext_tasklist: true,
        ext_superscript: true,
        ext_footnotes: true,
        .. ComrakOptions::default()
    };
    markdown_to_html(input, &options)
}
