use std::num::{NonZeroU8, NonZeroU32};
use std::collections::{HashSet, HashMap};
use serde::Deserialize;
use super::system::{Mods, System};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleKind {
    Free,
    Normal,
    Special,
}

#[derive(Debug)]
pub struct Role {
    pub name: String,
    pub info: Option<String>,
    pub kind: RoleKind,
    pub limit: Option<NonZeroU32>,
    pub level: NonZeroU8,
    pub level_max: Option<NonZeroU8>,
    pub trait_limit: u32,
    pub trait_balance: i32,
    pub provides: HashSet<String>,
    pub mods: Mods,
}

#[derive(Debug)]
pub struct Block {
    pub id: String,
    pub name: String,
    pub info: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug)]
pub struct Campaign {
    pub id: String,
    pub name: String,
    pub info: String,
    pub system: System,
    pub blocks: Vec<Block>,
    pub roles: HashMap<String, Role>,
}
