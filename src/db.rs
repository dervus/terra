use mysql_async::prelude::*;
use crate::MysqlConn;
use anyhow::Result;
use ring::digest::{SHA1, digest};
use crate::util::hexstring;

pub type MysqlResult<T> = Result<(MysqlConn, T)>;

#[derive(Debug)]
pub struct AccountInfo {
    pub id: u32,
    pub nick: String,
    pub access_level: u8,
}

impl AccountInfo {
    pub fn href(&self) -> String {
        format!("/users/{}", &self.nick)
    }
}

pub async fn fetch_account_info(conn: MysqlConn, id: u32) -> MysqlResult<Option<AccountInfo>> {
    const SQL: &'static str = "\
SELECT a.id, a.username, aa.gmlevel \
FROM account a LEFT JOIN account_access aa ON (a.id = aa.id AND aa.RealmID = -1) \
WHERE a.id = ?";

    let (conn, result) = conn.first_exec(SQL, (id,)).await?;
    let fields: Option<(u32, String, Option<u8>)> = result;

    Ok((conn, fields.map(|(id, nick, access_level)| {
        AccountInfo { id, nick, access_level: access_level.unwrap_or(0) }
    })))
}

pub async fn login_query(conn: MysqlConn, login: &str, password: &str) -> MysqlResult<Option<(u32, String)>> {
    const SQL: &'static str = "\
SELECT id, username, sha_pass_hash \
FROM account \
WHERE UPPER(username) = ? OR UPPER(email) = ?";

    let query = login.trim().to_uppercase();
    let (conn, result) = conn.first_exec(SQL, (&query, &query)).await?;
    let fields: Option<(u32, String, String)> = result;

    Ok((conn, result.and_then(|(id, nick, actual_passhash)| {
        let input_raw = format!("{}:{}", &nick, password.trim()).to_uppercase();
        let input_passhash = hexstring(digest(&SHA1, input_raw.as_bytes()).as_ref());

        if input_passhash == actual_passhash {
            Some((id, nick))
        } else {
            None
        }
    })))
}

use serde::{Serialize, Deserialize};
use crate::system::{EntityId, EntityIds, Gender, Campaign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
pub enum CharacterStatus {
    Pending,
    Rejected,
    Approved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub name_extra: String,
    pub role: Option<EntityId>,
    pub gender: Gender,
    pub race: EntityId,
    pub model: EntityId,
    pub class: EntityId,
    pub armorset: Option<EntityId>,
    pub weaponset: Option<EntityId>,
    pub traits: EntityIds,
    pub location: EntityId,
    pub description: String,
    pub comment: String,
    pub wants_loadup: bool,
    pub hidden: bool,
    pub status: Option<CharacterStatus>,
}

pub async fn insert_character(conn: MysqlConn, info: &CharCreationInfo, campaign: &Campaign) -> MysqlResult<u32> {
    const CHARACTER_SQL: &'static str = "\
INSERT INTO characters \
(account, name, nameExtra, gender, race, modelOverride, scaleOverride, class, level, maxLevel, money, map, zone, position_x, position_y, position_z, orientation, equipmentCache) \
VALUES \
(:account, :name, :gender, :race, :class, :level, :money, :map, :zone, :pos_x, :pos_y, :pos_z, :orientation)";

    const CHARACTER_INFO_SQL: &'static str = "\
INSERT INTO character_info \
(guid, role, race, model, class, armorset, weaponset, traits, location, description, comment, wantsLoadup, hidden, status) \
VALUES \
(:guid, :name_extra)";

    const CHARACTER_TEMPLATE_SQL: &'static str = "\
INSERT INTO character_template \
(guid, startingEquipment, startingItems, startingSkills, startingReputations, unlearnSpells, learnSpells) \
VALUES \
()";

    let role = info.role.and_then(|id| campaign.roles.get(&id));
    let race = campaign.system.races.get(&info.race).ok_or(anyhow::anyhow!("invalid race"))?;
    let class = campaign.system.classes.get(&info.class).ok_or(anyhow::anyhow!("invalid class"))?;
    let armorset = info.armorset.and_then(|id| campaign.system.armorsets.get(&id));
    let weaponset = info.weaponset.and_then(|id| campaign.system.weaponsets.get(&id));
    let traits = info.traits.iter().map(|id| campaign.system.traits.get(id));
    let location = campaign.system.locations.get(&info.location).ok_or(anyhow::anyhow!("invalid location"))?;

    let tx = conn.start_transaction(Default::default()).await?;

    let result = tx.prep_exec(CHARACTER_SQL, params! {
        "account" => 1,
        "name" => "Fargo",
        "gender" => match info.gender { Gender::Male => 0, Gender::Female => 1 },
        "race" => race.game_id,
        "class" => class.game_id,
        "level" => 1,
        "money" => 0,
        "map" => location.map,
        "zone" => location.zone,
        "pos_x" => location.position.0,
        "pos_y" => location.position.1,
        "pos_z" => location.position.2,
        "orientation" => location.orientation,
    }).await?;
    
    let guid = result.last_insert_id().unwrap_or(0) as u32;
    let tx = result.drop_result().await?;

    let tx = tx.drop_exec(CHARACTER_INFO_SQL, params! {
        "guid" => guid,
        "nameExtra" => "Just Fargo",
        "creationInfo" => serde_json::to_string(info)?,
    }).await?;

    let conn = tx.commit().await?;
    Ok((conn, guid))
}

pub async fn update_character(conn: MysqlConn, info: &CharCreationInfo, campaign: &Campaign) -> MysqlResult<u32> {
    const CHARACTER_SQL: &'static str = "\
UPDATE characters SET \
name = :name,
nameExtra = :name_extra,
gender,
race,
modelOverride,
scaleOverride,
class,
level,
maxLevel,
money,
map,
zone,
position_x,
position_y,
position_z,
orientation,
equipmentCache) \
";

    const CHARACTER_INFO_SQL: &'static str = "\
UPDATE character_info SET \
(guid, role, race, model, class, armorset, weaponset, traits, location, description, comment, wantsLoadup, hidden, status) \
";

    const CHARACTER_TEMPLATE_SQL: &'static str = "\
INSERT INTO character_template \
(guid, startingEquipment, startingItems, startingSkills, startingReputations, unlearnSpells, learnSpells) \
VALUES \
()";

    todo!()
}
