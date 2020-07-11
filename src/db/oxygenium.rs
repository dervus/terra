use std::collections::{HashMap, HashSet};
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use serde::{Serialize, Deserialize};
use bitflags::bitflags;
use mysql_async::prelude::*;
use crate::errors::{TerraError, TerraResult};
use crate::system::{EntityId, EntityIds, Campaign, Gender, ExtraMods};
use crate::util;
use super::{MysqlConn, MysqlResult, AccountInfo};

lazy_static! {
    static ref NAME_REGEX: Regex =
        RegexBuilder::new(r"[а-я]{2,12}")
        .case_insensitive(true)
        .build()
        .unwrap();
    static ref NAME_EXTRA_REGEX: Regex =
        RegexBuilder::new(r"[а-я \-']{0,20}")
        .case_insensitive(true)
        .build()
        .unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CharacterStatus {
    Pending,
    Reviewed,
    Rejected,
    Approved,
}

impl CharacterStatus {
    pub fn from_str(from: &str) -> TerraResult<Self> {
        match from {
            "pending" => Ok(Self::Pending),
            "reviewed" => Ok(Self::Reviewed),
            "rejected" => Ok(Self::Rejected),
            "approved" => Ok(Self::Approved),
            _ => Err(TerraError::InvalidInput),
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Reviewed => "reviewed",
            Self::Rejected => "rejected",
            Self::Approved => "approved",
        }
    }
}

bitflags! {
    pub struct AtLoginFlags: u16 {
        const RENAME            = 0x001;
        const RESET_SPELLS      = 0x002;
        const RESET_TALENTS     = 0x004;
        const CUSTOMIZE         = 0x008;
        const RESET_PET_TALENTS = 0x010;
        const FIRST_LOGIN       = 0x020;
        const CHANGE_FACTION    = 0x040;
        const CHANGE_RACE       = 0x080;
        const RESURRECT         = 0x100;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub role: Option<EntityId>,
    pub race: EntityId,
    pub gender: Gender,
    pub model: Option<EntityId>,
    pub class: EntityId,
    pub armorset: Option<EntityId>,
    pub weaponset: Option<EntityId>,
    pub traits: EntityIds,
    pub location: EntityId,
    pub name: String,
    pub name_extra: Option<String>,
    pub description: String,
    pub comment: String,
    pub wants_loadup: bool,
    pub hidden: bool,
}

impl Character {
    

    pub fn validate(&self, _campaign: &Campaign) -> TerraResult<()> {
        if !NAME_REGEX.is_match(&self.name) {
            return Err(TerraError::InvalidInput);
        }
        if let Some(name_extra) = &self.name_extra {
            if !NAME_EXTRA_REGEX.is_match(name_extra) {
                return Err(TerraError::InvalidInput);
            }
        }
        Ok(())
    }
}

pub struct CharacterInfo {
    pub guid: u32,
    pub name: String,
    pub name_extra: Option<String>,
    pub role: Option<EntityId>,
    pub gender: Gender,
    pub race: EntityId,
    pub class: EntityId,
    pub wants_loadup: bool,
    pub hidden: bool,
}

pub async fn list_characters(conn: MysqlConn, campaign: &Campaign) -> MysqlResult<Vec<CharacterInfo>> {
    const SQL: &'static str = "\
SELECT c.guid, c.name, c.nameExtra, f.role, c.gender, f.race, f.class, f.wantsLoadup, f.hidden
FROM characters c JOIN character_form f USING (guid)
WHERE f.campaign = ?
ORDER BY guid";

    conn.prep_exec(SQL, (&campaign.id,)).await?.map_and_drop(|row| {
        let row: (u32, String, Option<String>, Option<EntityId>, u8, EntityId, EntityId, bool, bool) = FromRow::from_row(row);
        let (guid, name, name_extra, role, gender, race, class, wants_loadup, hidden) = row;
        CharacterInfo {
            guid,
            name,
            name_extra,
            role,
            gender: if gender != 0 { Gender::Female } else { Gender::Male },
            race,
            class,
            wants_loadup,
            hidden
        }
    }).await.map_err(|e| e.into())
}

pub async fn fetch_character(conn: MysqlConn, guid: u32) -> MysqlResult<Character> {
    const SQL: &'static str = "\
SELECT f.role, f.race, c.gender, f.model, f.class, f.armorset, f.weaponset, f.traits, f.location, c.name, c.nameExtra, f.description, f.comment, f.wantsLoadup, f.hidden
FROM characters c JOIN character_form f USING (guid)
WHERE guid = ?";

    let (conn, result) = conn.first_exec(SQL, (guid,)).await?;
    let fields: Option<(String,)> = result;
    todo!()
}

pub async fn check_character_name(conn: MysqlConn, name: &str) -> MysqlResult<bool> {
    const SQL: &'static str = "SELECT COUNT(*) FROM characters WHERE name = ?";

    let prepared_name = util::prepare_name(name);
    let (conn, result): (MysqlConn, Option<usize>) = conn.first_exec(SQL, (prepared_name,)).await?;
    let count = result.unwrap_or(0);

    Ok((conn, count != 0))
}

pub async fn insert_character(conn: MysqlConn, account: &AccountInfo, campaign: &Campaign, info: &Character) -> MysqlResult<u32> {
    const CHARACTER_SQL: &'static str = "\
INSERT INTO characters (account, at_login, cinematic, name, nameExtra, gender, race, modelOverride, modelScale, speedMod, class, level, maxLevel, money, map, zone, position_x, position_y, position_z, orientation, bannedSpells, innateSpells, startingSkills, startingEquip, startingItems)
VALUES (:account, :atloginflags, '1', :name, :nameextra, :gender, :race, :modeloverride, :modelscale, :speedmod, :class, :level, :maxlevel, :money, :map, :zone, :posx, :posy, :posz, :orientation, :bannedspells, :innatespells, :startingskills, :startingequip, :startingitems)";

    const CHARACTER_FORM_SQL: &'static str = "\
INSERT INTO character_form (guid, campaign, role, race, model, class, armorset, weaponset, traits, location, description, comment, wantsLoadup, hidden, status)
VALUES (:guid, :campaign, :role, :race, :model, :class, :armorset, :weaponset, :traits, :location, :description, :comment, :wantsloadup, :hidden, :status)";

    const CHARACTER_HOMEBIND_SQL: &'static str = "\
INSERT INTO character_homebind (guid, mapId, zoneId, posX, posY, posZ)
VALUES (:guid, :map, :zone, :posx, :posy, :posz)";

    fn hashset_to_string(input: &HashSet<u32>) -> String {
        input.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")
    }

    fn hashmap_to_string(input: &HashMap<u32, i16>) -> String {
        input.iter().map(|x| format!("{} {}", x.0, std::cmp::max(x.1, &0))).collect::<Vec<_>>().join(" ")
    }

    let role = info.role.as_ref().and_then(|id| campaign.roles.get(id));
    let race = campaign.system.races.get(&info.race).ok_or(anyhow::anyhow!("invalid race"))?;
    let model = info.model.as_ref().and_then(|id| race.models.get(id));
    let class = campaign.system.classes.get(&info.class).ok_or(anyhow::anyhow!("invalid class"))?;
    let armorset = info.armorset.as_ref().and_then(|id| campaign.system.armorsets.get(id));
    let weaponset = info.weaponset.as_ref().and_then(|id| campaign.system.weaponsets.get(id));
    let traits = info.traits.iter().flat_map(|id| campaign.system.traits.get(id)).collect::<Vec<_>>();
    let location = campaign.system.locations.get(&info.location).ok_or(anyhow::anyhow!("invalid location"))?;

    let at_login_flags = if model.map(|m| m.customizable).unwrap_or(true) {
        AtLoginFlags::FIRST_LOGIN | AtLoginFlags::CUSTOMIZE
    } else {
        AtLoginFlags::FIRST_LOGIN
    };

    let mut mods = ExtraMods::default();
    if let Some(e) = role { mods.merge(&e.mods) }
    for e in &traits { mods.merge(&e.mods) }
    mods.merge(&location.mods);

    let equip_string = [
        armorset.and_then(|e| e.head).unwrap_or(0),
        armorset.and_then(|e| e.neck).unwrap_or(0),
        armorset.and_then(|e| e.shoulders).unwrap_or(0),
        armorset.and_then(|e| e.body).unwrap_or(0),
        armorset.and_then(|e| e.chest).unwrap_or(0),
        armorset.and_then(|e| e.waist).unwrap_or(0),
        armorset.and_then(|e| e.legs).unwrap_or(0),
        armorset.and_then(|e| e.feet).unwrap_or(0),
        armorset.and_then(|e| e.wrists).unwrap_or(0),
        armorset.and_then(|e| e.hands).unwrap_or(0),
        armorset.and_then(|e| e.fingers.iter().nth(0).cloned()).unwrap_or(0),
        armorset.and_then(|e| e.fingers.iter().nth(1).cloned()).unwrap_or(0),
        armorset.and_then(|e| e.trinkets.iter().nth(0).cloned()).unwrap_or(0),
        armorset.and_then(|e| e.trinkets.iter().nth(1).cloned()).unwrap_or(0),
        armorset.and_then(|e| e.back).unwrap_or(0),
        weaponset.and_then(|e| e.mainhand).unwrap_or(0),
        weaponset.and_then(|e| e.offhand).unwrap_or(0),
        weaponset.and_then(|e| e.ranged).unwrap_or(0),
        armorset.and_then(|e| e.tabard).unwrap_or(0),
        armorset.and_then(|e| e.bags.iter().nth(0).cloned()).unwrap_or(0),
        armorset.and_then(|e| e.bags.iter().nth(1).cloned()).unwrap_or(0),
        armorset.and_then(|e| e.bags.iter().nth(2).cloned()).unwrap_or(0),
        armorset.and_then(|e| e.bags.iter().nth(3).cloned()).unwrap_or(0),
    ].iter().map(|item_id| item_id.to_string()).collect::<Vec<_>>().join(" ");

    let tx = conn.start_transaction(Default::default()).await?;

    let result = tx.prep_exec(CHARACTER_SQL, params! {
        "account" => account.id,
        "atloginflags" => at_login_flags.bits(),
        "name" => info.name.clone(),
        "nameextra" => info.name_extra.clone(),
        "gender" => match info.gender { Gender::Male => 0, Gender::Female => 1 },
        "race" => race.game_id,
        "modeloverride" => model.map(|e| e.display_id).unwrap_or(0),
        "modelscale" => model.map(|e| e.scale).unwrap_or(1.0f32),
        "speedmod" => model.map(|e| e.speed).unwrap_or(0.0f32),
        "class" => class.game_id,
        "level" => role.and_then(|e| e.level).map(|l| l.0).unwrap_or(campaign.manifest.level_range.0),
        "maxlevel" => role.and_then(|e| e.level).map(|l| l.0).unwrap_or(campaign.manifest.level_range.1),
        "money" => std::cmp::max(mods.money, 0),
        "map" => location.map,
        "zone" => location.zone,
        "posx" => location.position.0,
        "posy" => location.position.1,
        "posz" => location.position.2,
        "orientation" => location.orientation,
        "bannedspells" => hashset_to_string(&mods.spells_banned),
        "innatespells" => hashset_to_string(&mods.spells),
        "startingskills" => hashmap_to_string(&mods.skills),
        "startingitems" => hashmap_to_string(&mods.items),
        "startingequip" => equip_string,
    }).await?;
    
    let guid = result.last_insert_id().ok_or(anyhow::anyhow!("unable to get last insert id"))? as u32;
    let tx = result.drop_result().await?;

    let tx = tx.drop_exec(CHARACTER_FORM_SQL, params! {
        "guid" => guid,
        "campaign" => &campaign.id,
        "role" => &info.role,
        "race" => &info.race,
        "model" => &info.model,
        "class" => &info.class,
        "armorset" => &info.armorset,
        "weaponset" => &info.weaponset,
        "traits" => info.traits.iter().cloned().collect::<Vec<_>>().join(" "),
        "location" => &info.location,
        "description" => &info.description,
        "comment" => &info.comment,
        "wantsloadup" => info.wants_loadup,
        "hidden" => info.hidden,
        "status" => CharacterStatus::Pending.as_str(),
    }).await?;

    let tx = tx.drop_exec(CHARACTER_HOMEBIND_SQL, params! {
        "guid" => guid,
        "map" => location.map,
        "zone" => location.zone,
        "posx" => location.position.0,
        "posy" => location.position.1,
        "posz" => location.position.2,
    }).await?;

    let conn = tx.commit().await?;
    Ok((conn, guid))
}

// pub async fn update_character(conn: MysqlConn, campaign: &Campaign, info: &Character) -> MysqlResult<u32> {
//     const CHARACTER_SQL: &'static str = "\
// UPDATE characters SET \
// name = :name,
// nameExtra = :name_extra,
// gender,
// race,
// modelOverride,
// scaleOverride,
// class,
// level,
// maxLevel,
// money,
// map,
// zone,
// position_x,
// position_y,
// position_z,
// orientation,
// equipmentCache) \
// ";

//     const CHARACTER_FORM_SQL: &'static str = "\
// UPDATE character_form SET \
// (guid, campaign, role, race, model, class, armorset, weaponset, traits, location, description, comment, wantsLoadup, hidden, status) \
// ";

//     todo!()
// }
