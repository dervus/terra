use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs::{File, read_to_string};
use std::io::BufReader;
use anyhow::{anyhow, Result as AnyhowResult};
use log::info;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use comrak::{ComrakOptions, markdown_to_html};
use crate::system::System;
use crate::campaign::{Campaign, Template, Block, Role};

pub fn load_campaign<P: AsRef<Path>>(base_path: P, id: &str) -> AnyhowResult<Campaign> {
    #[derive(Deserialize)]
    struct Manifest {
        name: String,
        trait_limit: u32,
        trait_balance: i32,
        default_tags: HashSet<String>,
    }
    #[derive(Clone, Default, Deserialize)]
    struct BlockDef {
        id: Option<String>,
        name: String,
        #[serde(default)] description: Option<String>,
        #[serde(default)] provides: HashSet<String>,
        roles: Vec<RoleDef>,
    }
    #[derive(Clone, Default, Deserialize)]
    struct RoleDef {
        id: Option<String>,
        #[serde(default)] provides: HashSet<String>,
        #[serde(flatten)] info: Role,
    }

    let path = base_path.into().join("campaigns").join(id);

    let manifest: Manifest =
        load_yaml_required(&path.join("manifest.yml"))?;
    let system =
        load_system(&[path.join("system.yml"), path.join("system/**/*.yml")])?;
    let templates: HashMap<String, Template> =
        load_yaml_optional(&path.join("templates.yml"))?.unwrap_or_default();
    let roles_src: Vec<BlockDef> =
        load_yaml_optional(&path.join("roles.yml"))?.unwrap_or_default();
    let description =
        load_markdown(&path.join("description.md"))?;

    let blocks: Vec<Block> = roles_src.iter().map(|b| Block {
        id: b.id.clone(),
        name: b.name.clone(),
        description: b.description.clone(),
        roles: b.roles.iter().map(|r| format!("{}_{}", &b.id, &r.id)).collect()
    }).collect();
    
    let roles: HashMap<String, Role> = roles_src
        .iter()
        .flat_map(|b| b.roles.iter().map(|r| (format!("{}_{}", &b.id, &r.id), r.info.clone())).collect::<Vec<_>>())
        .collect();

    Ok(Campaign {
        id: id.to_owned(),
        name: manifest.name,
        description,
        trait_limit: manifest.trait_limit,
        trait_balance: manifest.trait_balance,
        default_tags: manifest.default_tags,
        system,
        blocks,
        roles
    })
}

pub fn load_system<I>(patterns: I) -> AnyhowResult<System>
where
    I: IntoIterator,
    I::Item: AsRef<Path>
{
    let mut system = System::default();
    for pattern in patterns {
        let pattern_str = pattern.to_str().ok_or_else(|| anyhow!("Invalid path: {:?}", pattern))?;
        for glob_result in glob::glob(pattern_str)? {
            let path = glob_result?;
            system.merge_in(&load_yaml_required(&path)?);
        }
    }
    Ok(system)
}

fn load_yaml_required<T: DeserializeOwned>(path: &Path) -> AnyhowResult<T> {
    info!("Loading system file {:?}", path);
    let file = File::open(path)?;
    let yaml: T = serde_yaml::from_reader(BufReader::new(file))?;
    Ok(yaml)
}

fn load_yaml_optional<T: DeserializeOwned>(path: &Path) -> AnyhowResult<Option<T>> {
    if path.exists() {
        load_yaml_required(path).map(Some)
    } else {
        Ok(None)
    }
}

fn load_markdown(path: &Path) -> AnyhowResult<String> {
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
