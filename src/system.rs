use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs::{File, read_to_string};
use std::io::BufReader;
use std::fmt;

use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use comrak::{ComrakOptions, markdown_to_html};

pub type Result<T> = std::result::Result<T, anyhow::Error>;
pub type EntityId = String;
pub type EntityIds = HashSet<EntityId>;
pub type EntityMap<T> = HashMap<EntityId, T>;

// =============================================================================
// BASIC TYPES
//
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeclinedName {
    Uniform(String),
    Declined(String, String),
}

impl DeclinedName {
    pub fn get(&self, gender: Gender) -> &str {
        match self {
            DeclinedName::Uniform(uniform) => &uniform,
            DeclinedName::Declined(male, female) => match gender {
                Gender::Male => &male,
                Gender::Female => &female
            }
        }
    }

    pub fn male(&self) -> &str { self.get(Gender::Male) }
    pub fn female(&self) -> &str { self.get(Gender::Female) }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GenderFilter {
    Any,
    MaleOnly,
    FemaleOnly,
}

impl GenderFilter {
    pub fn apply(&self, gender: Gender) -> bool {
        match self {
            GenderFilter::Any => true,
            GenderFilter::MaleOnly => gender == Gender::Male,
            GenderFilter::FemaleOnly => gender == Gender::Female
        }
    }
}

impl Default for GenderFilter {
    fn default() -> Self { GenderFilter::Any }
}

impl fmt::Display for GenderFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenderFilter::Any => f.write_str("any"),
            GenderFilter::MaleOnly => f.write_str("male-only"),
            GenderFilter::FemaleOnly => f.write_str("female-only")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntiryFilter {
    Pass,
    Allow(EntityIds),
    Deny(EntityIds),
}

impl EntiryFilter {
    pub fn apply(&self, id: &str) -> bool {
        match self {
            EntiryFilter::Pass => true,
            EntiryFilter::Allow(s) => s.contains(id),
            EntiryFilter::Deny(s) => !s.contains(id),
        }
    }
}

impl Default for EntiryFilter {
    fn default() -> Self { EntiryFilter::Pass }
}

impl fmt::Display for EntiryFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntiryFilter::Pass => {
                f.write_str("pass")
            }
            EntiryFilter::Allow(ids) => {
                f.write_str("allow:")?;
                for id in ids {
                    f.write_str(" ")?;
                    f.write_str(id)?;
                }
                Ok(())
            }
            EntiryFilter::Deny(ids) => {
                f.write_str("deny:")?;
                for id in ids {
                    f.write_str(" ")?;
                    f.write_str(id)?;
                }
                Ok(())
            }
        }
    }
}

// =============================================================================
// ENTITY TYPES
//
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtraMods {
    #[serde(default)] pub spells: HashSet<u32>,
    #[serde(default)] pub unlearn: HashSet<u32>,
    #[serde(default)] pub skills: HashMap<u32, i16>,
    #[serde(default)] pub items: HashMap<u32, i16>,
    #[serde(default)] pub money: i32,
}

impl ExtraMods {
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
        self.unlearn.extend(&other.unlearn);
        merge_amounts(&mut self.skills, &other.skills);
        merge_amounts(&mut self.items, &other.items);
        self.money += other.money;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Race {
    pub name: DeclinedName,
    pub game_id: u32,
    #[serde(default)] pub gender: GenderFilter,
    #[serde(default)] pub models: EntityMap<Model>,
}

fn default_model_scale() -> f32 { 1.0 }
fn default_model_speed() -> f32 { 1.0 }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub name: String,
    pub display_id: u32,
    #[serde(default)] pub gender: GenderFilter,
    #[serde(default = "default_model_scale")] pub scale: f32,
    #[serde(default = "default_model_speed")] pub speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Class {
    pub name: DeclinedName,
    pub game_id: u32,
    #[serde(default)] pub gender: GenderFilter,
    #[serde(default)] pub races: EntiryFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmorSet {
    pub name: String,
    #[serde(default)] pub description: Option<String>,
    
    #[serde(default)] pub gender: GenderFilter,
    #[serde(default)] pub races: EntiryFilter,
    #[serde(default)] pub classes: EntiryFilter,
    
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
    pub name: String,
    #[serde(default)] pub description: Option<String>,

    #[serde(default)] pub gender: GenderFilter,
    #[serde(default)] pub races: EntiryFilter,
    #[serde(default)] pub classes: EntiryFilter,
    
    #[serde(default)] pub mainhand: Option<u32>,
    #[serde(default)] pub offhand: Option<u32>,
    #[serde(default)] pub ranged: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trait {
    pub name: String,
    #[serde(default)] pub description: Option<String>,

    #[serde(default)] pub gender: GenderFilter,
    #[serde(default)] pub races: EntiryFilter,
    #[serde(default)] pub classes: EntiryFilter,
    #[serde(default)] pub group: String,
    #[serde(default)] pub unique: bool,
    
    #[serde(default)] pub cost: i32,
    #[serde(default, flatten)] pub mods: ExtraMods,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub name: String,
    #[serde(default)] pub description: Option<String>,

    #[serde(default)] pub gender: GenderFilter,
    #[serde(default)] pub races: EntiryFilter,
    #[serde(default)] pub classes: EntiryFilter,

    pub map: u32,
    pub zone: u32,
    pub position: (f32, f32, f32),
    pub orientation: f32,

    #[serde(default, flatten)] pub mods: ExtraMods,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
    pub name: String,
    #[serde(default)] pub kind: RoleKind,
    #[serde(default)] pub limit: Option<u32>,
    #[serde(default)] pub gender: GenderFilter,
    #[serde(default)] pub races: EntiryFilter,
    #[serde(default)] pub classes: EntiryFilter,
    #[serde(default)] pub armorsets: EntiryFilter,
    #[serde(default)] pub weaponsets: EntiryFilter,
    #[serde(default)] pub traits: EntiryFilter,
    #[serde(default)] pub locations: EntiryFilter,
    #[serde(default, flatten)] pub mods: ExtraMods,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub id: EntityId,
    pub name: String,
    #[serde(default)] pub description: Option<String>,
    pub roles: Vec<EntityId>,
}

// =============================================================================
// TOP-LEVEL TYPES
//
#[derive(Debug, Clone)]
pub struct Campaign {
    pub manifest: Manifest,
    pub index_page: String,
    pub system: System,
    pub blocks: Vec<Block>,
    pub roles: EntityMap<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub max_traits: u32,
    pub max_traits_cost: i32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct System {
    #[serde(default)] pub races: HashMap<EntityId, Race>,
    #[serde(default)] pub classes: HashMap<EntityId, Class>,
    #[serde(default)] pub armorsets: HashMap<EntityId, ArmorSet>,
    #[serde(default)] pub weaponsets: HashMap<EntityId, WeaponSet>,
    #[serde(default)] pub traits: HashMap<EntityId, Trait>,
    #[serde(default)] pub locations: HashMap<EntityId, Location>,
}

impl System {
    pub fn merge(&mut self, other: &System) {
        fn walk<T: Clone>(into: &mut EntityMap<T>, from: &EntityMap<T>) {
            for (key, value) in from {
                if !into.contains_key(key) {
                    into.insert(key.clone(), value.clone());
                }
            }
        }
        walk(&mut self.races, &other.races);
        walk(&mut self.classes, &other.classes);
        walk(&mut self.armorsets, &other.armorsets);
        walk(&mut self.weaponsets, &other.weaponsets);
        walk(&mut self.traits, &other.traits);
        walk(&mut self.locations, &other.locations);
    }
}

pub fn load_shared_system<P: Into<PathBuf>>(path: P) -> Result<System> {
    let path = path.into().join("shared");

    let races = load_yaml_optional(&path.join("races.yml"))?;
    let classes = load_yaml_optional(&path.join("classes.yml"))?;
    let armorsets = load_yaml_optional(&path.join("armorsets.yml"))?;
    let weaponsets = load_yaml_optional(&path.join("weaponsets.yml"))?;
    let traits = load_yaml_optional(&path.join("traits.yml"))?;
    let locations = load_yaml_optional(&path.join("locations.yml"))?;

    Ok(System { races, classes, armorsets, weaponsets, traits, locations })
}

pub fn load_campaign<P: Into<PathBuf>>(path: P, id: &str, shared_system: Option<&System>) -> Result<Campaign> {
    #[derive(Clone, Deserialize)]
    struct BlockDef {
        id: EntityId,
        name: String,
        #[serde(default)] description: Option<String>,
        roles: Vec<RoleDef>,
    }
    #[derive(Clone, Deserialize)]
    struct RoleDef {
        id: EntityId,
        #[serde(flatten)] info: Role,
    }

    let campaign_path = path.into().join("campaigns").join(id);
    
    let manifest: Manifest = load_yaml_required(&campaign_path.join("manifest.yml"))?;
    let mut system: System = load_yaml_optional(&campaign_path.join("system.yml"))?;
    let roles_data: Vec<BlockDef> = load_yaml_optional(&campaign_path.join("roles.yml"))?;
    let index_page = load_markdown(&campaign_path.join("index.md"))?;

    if let Some(some_shared_system) = shared_system {
        system.merge(some_shared_system);
    }

    let blocks: Vec<Block> = roles_data.iter().map(|b| Block {
        id: b.id.clone(),
        name: b.name.clone(),
        description: b.description.clone(),
        roles: b.roles.iter().map(|r| format!("{}_{}", &b.id, &r.id)).collect()
    }).collect();
    
    let roles: EntityMap<Role> = roles_data
        .iter()
        .flat_map(|b| b.roles.iter().map(|r| (format!("{}_{}", &b.id, &r.id), r.info.clone())).collect::<Vec<_>>())
        .collect();

    Ok(Campaign {
        manifest,
        index_page,
        system,
        blocks,
        roles
    })
}

fn load_yaml_required<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let file = File::open(path)?;
    let yaml: T = serde_yaml::from_reader(BufReader::new(file))?;
    Ok(yaml)
}

fn load_yaml_optional<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    if path.exists() {
        load_yaml_required(path)
    } else {
        Ok(Default::default())
    }
}

fn load_markdown(path: &Path) -> Result<String> {
    let source = read_to_string(path)?;
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
    Ok(markdown_to_html(&source, &options))
}
