pub mod campaign;
pub mod system;
pub mod tags;

use std::{collections::HashMap, num::NonZeroU32, path::Path};
use log::{debug, trace, warn};
use serde::Deserialize;
use crate::util;
use self::{
    campaign::{Block, Campaign, Role, RoleKind},
    system::{Mods, System},
    tags::Tags,
};

pub fn load_campaign<P: AsRef<Path>>(
    campaign_path: P,
    assets_path: Option<P>,
) -> anyhow::Result<Campaign> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ManifestFile {
        name: String,
        role_template: RoleTemplate,
        blocks: Vec<BlockDef>,
    }
    #[derive(Clone, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RoleTemplate {
        kind: RoleKind,
        #[serde(default)]
        provides: Tags,
        #[serde(flatten)]
        mods: Mods,
    }
    #[derive(Clone, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct BlockDef {
        #[serde(default)]
        id: Option<String>,
        name: String,
        #[serde(default)]
        info: Option<String>,
        #[serde(default)]
        provides: Tags,
        #[serde(flatten)]
        mods: Mods,
        roles: Vec<RoleDef>,
    }
    #[derive(Clone, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RoleDef {
        #[serde(default)]
        id: Option<String>,
        name: String,
        #[serde(default)]
        info: Option<String>,
        #[serde(default)]
        kind: Option<RoleKind>,
        #[serde(default)]
        limit: Option<NonZeroU32>,
        #[serde(default)]
        provides: Tags,
        #[serde(flatten)]
        mods: Mods,
    }

    let campaign_path = campaign_path.as_ref();

    let manifest: ManifestFile = util::load_yaml(&campaign_path.join("manifest.yml"))?;
    let info = util::load_markdown(&campaign_path.join("info.md"))?;
    let system = load_system(&[
        campaign_path.join("system.yml"),
        campaign_path.join("system"),
    ])?;

    if let Some(base_path) = assets_path {
        for info in system.info_iter() {
            if let Some(preview_path) = &info.preview {
                if base_path.as_ref().join(preview_path).exists() {
                    debug!("Found preview file {:?}", preview_path);
                } else {
                    warn!("Missing preview file {:?}", preview_path);
                }
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
            roles: Vec::new(),
        };
        for role in block.roles {
            let id = format!(
                "{}_{}",
                &compiled_block.id,
                role.id.unwrap_or(util::name_to_id(&role.name))
            );
            let mut compiled_role = Role {
                name: role.name,
                info: role.info,
                kind: role.kind.unwrap_or(manifest.role_template.kind),
                limit: role.limit,
                provides: role.provides,
                mods: role.mods,
            };
            compiled_role.mods.merge_in(&manifest.role_template.mods);
            compiled_role.mods.merge_in(&block.mods);
            compiled_role
                .provides
                .merge_in(&manifest.role_template.provides);
            compiled_role.provides.merge_in(&block.provides);
            compiled_role
                .provides
                .add(format!("block/{}", &compiled_block.id), 1);
            compiled_role.provides.add(format!("role/{}", &id), 1);
            compiled_block.roles.push(id.clone());
            resolved_roles.insert(id, compiled_role);
        }
        resolved_blocks.push(compiled_block);
    }

    Ok(Campaign {
        name: manifest.name,
        info,
        system_view: system.view(),
        system,
        blocks: resolved_blocks,
        roles: resolved_roles,
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
