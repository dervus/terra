pub mod tags;
pub mod system;
pub mod campaign;

use std::num::{NonZeroU8, NonZeroU32};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use log::{info, warn, debug, trace};
use serde::Deserialize;
use self::system::{System, Mods};
use self::campaign::{Campaign, Block, Role, RoleKind};
use crate::util;

pub fn load_campaign<P: AsRef<Path>>(base_path: P, id: &str) -> anyhow::Result<Campaign> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ManifestFile {
        name: String,
        role_default: RoleKind,
        level: NonZeroU8,
        #[serde(default)] level_max: Option<NonZeroU8>,
        trait_limit: u32,
        trait_balance: i32,
        blocks: Vec<BlockDef>,
    }
    #[derive(Clone, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct BlockDef {
        #[serde(default)] id: Option<String>,
        name: String,
        #[serde(default)] info: Option<String>,
        #[serde(default)] provides: HashSet<String>,
        roles: Vec<RoleDef>,
    }
    #[derive(Clone, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RoleDef {
        #[serde(default)] id: Option<String>,
        name: String,
        #[serde(default)] info: Option<String>,
        #[serde(default)] kind: Option<RoleKind>,
        #[serde(default)] limit: Option<NonZeroU32>,
        #[serde(default)] level: Option<NonZeroU8>,
        #[serde(default)] level_max: Option<NonZeroU8>,
        #[serde(default)] trait_limit: Option<u32>,
        #[serde(default)] trait_balance: Option<i32>,
        #[serde(default)] provides: HashSet<String>,
        #[serde(flatten)] mods: Mods,
    }

    let campaign_path = base_path.as_ref().join("campaigns").join(id);
    let assets_path = base_path.as_ref().join("assets");

    let manifest: ManifestFile = util::load_yaml(&campaign_path.join("manifest.yml"))?;
    let info = util::load_markdown(&campaign_path.join("info.md"))?;
    let system = load_system(&[campaign_path.join("system.yml"), campaign_path.join("system")])?;

    for info in system.info_iter() {
        if let Some(path) = &info.preview {
            if assets_path.join(path).exists() {
                debug!("Found preview file {:?}", path);
            } else {
                warn!("Missing preview file {:?}", path);
            }
        }
    }

    let mut resolved_blocks = Vec::new();
    let mut resolved_roles = HashMap::new();

    for block in manifest.blocks {
        let mut compiled_block = Block {
            id: block.id.unwrap_or(util::name_to_id(&block.name)),
            name: block.name,
            info: block.info,
            roles: Vec::new()
        };
        for role in block.roles {
            let id = format!("{}_{}", &compiled_block.id, role.id.unwrap_or(util::name_to_id(&role.name)));
            let mut compiled_role = Role {
                name: role.name,
                info: role.info,
                kind: role.kind.unwrap_or(manifest.role_default),
                limit: role.limit,
                level: role.level.unwrap_or(manifest.level),
                level_max: role.level_max.or(manifest.level_max),
                trait_limit: role.trait_limit.unwrap_or(manifest.trait_limit),
                trait_balance: role.trait_balance.unwrap_or(manifest.trait_balance),
                provides: role.provides,
                mods: role.mods
            };
            compiled_role.provides.extend(block.provides.iter().cloned());
            compiled_role.provides.insert(format!("block:{}", &compiled_block.id));
            compiled_role.provides.insert(format!("role:{}", &id));
            compiled_block.roles.push(id.clone());
            resolved_roles.insert(id, compiled_role);
        }
        resolved_blocks.push(compiled_block);
    }

    Ok(Campaign {
        id: id.to_owned(),
        name: manifest.name,
        info,
        system,
        blocks: resolved_blocks,
        roles: resolved_roles
    })
}

pub fn load_system<I>(paths: I) -> anyhow::Result<System>
where
    I: IntoIterator,
    I::Item: AsRef<Path>,
{
    fn load_file(system: &mut System, file_path: &Path) -> anyhow::Result<()> {
        let file = util::load_yaml(&file_path)?;
        system.merge_in(&file);
        Ok(())
    };
    fn load_dir(system: &mut System, dir_path: &Path) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(dir_path)? {
            let subpath = entry?.path();
            load(system, &subpath)?;
        }
        Ok(())
    };
    fn load(system: &mut System, path: &Path) -> anyhow::Result<()> {
        trace!("Looking at {:?}", path);
        if path.is_dir() {
            load_dir(system, path)?;
        } else if path.extension().map(|ext| ext == "yml").unwrap_or(false) {
            load_file(system, path)?;   
        } else {
            debug!("Skipping non-system file {:?}", path);
        }
        Ok(())
    }

    let mut system = System::new();
    
    for path in paths {
        if path.as_ref().exists() {
            load(&mut system, path.as_ref())?;
        }
    }

    Ok(system)
}
