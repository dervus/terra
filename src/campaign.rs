use std::num::NonZeroU8;
use std::collections::{HashSet, HashMap};
use serde::Deserialize;
use crate::tags::Constraint;
use crate::system::{Mods, System};

#[derive(Debug, Clone, Deserialize)]
pub struct Template {
    starting_level: NonZeroU8,
    #[serde(default)] max_level: Option<NonZeroU8>,
    #[serde(default)] cases: Vec<TemplateCase>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateCase {
    #[serde(default, rename = "if")] cond: Option<Constraint>,
    #[serde(flatten)] mods: Mods,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleKind {
    Free,
    Normal,
    Special,
}

impl Default for RoleKind {
    fn default() -> Self { Self::Normal }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Role {
    pub name: String,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub kind: RoleKind,
    #[serde(default)] pub limit: Option<u32>,
    #[serde(default)] pub provides: HashSet<String>,
    #[serde(flatten)] pub mods: Mods,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Campaign {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trait_limit: u32,
    pub trait_balance: i32,
    pub default_tags: HashSet<String>,
    pub system: System,
    pub templates: HashMap<String, Template>,
    pub blocks: Vec<Block>,
    pub roles: HashMap<String, Role>,
}
