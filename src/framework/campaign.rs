use std::{collections::HashMap, num::NonZeroU32};
use serde::{Deserialize, Serialize};
use super::{
    system::{Mods, System, SystemView},
    tags::Tags,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleKind {
    Free,
    Normal,
    Special,
}

#[derive(Debug, Serialize)]
pub struct Role {
    pub name: String,
    pub info: Option<String>,
    pub kind: RoleKind,
    pub limit: Option<NonZeroU32>,
    pub provides: Tags,
    pub mods: Mods,
}

#[derive(Debug, Serialize)]
pub struct Block {
    pub id: String,
    pub name: String,
    pub info: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Campaign {
    pub name: String,
    pub info: String,
    pub system: System,
    pub system_view: SystemView,
    pub blocks: Vec<Block>,
    pub roles: HashMap<String, Role>,
}
