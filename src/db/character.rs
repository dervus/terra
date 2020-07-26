use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet},
    num::NonZeroU32,
};
use bitflags::bitflags;
use fallible_iterator::{convert as fall_iter, FallibleIterator};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use sqlx::{mysql::MySqlPool, prelude::*};
use crate::{
    util,
    error::{AppError, AppResult},
    framework::{
        campaign::{Campaign, RoleKind},
        system::{Armor, Mods, Weapon},
        tags::Tags,
    }
};

const LEVEL_MIN: i32 = 1;
const LEVEL_MAX: i32 = 80;
const EQUIP_SLOTS: usize = 23;
type EquipArray = [Option<NonZeroU32>; EQUIP_SLOTS];

static NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"[а-я]{2,12}")
        .case_insensitive(true)
        .build()
        .unwrap()
});
static NAME_EXTRA_REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"[а-я \-']{0,20}")
        .case_insensitive(true)
        .build()
        .unwrap()
});

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

#[derive(Serialize, Deserialize, FromRow)]
#[serde(deny_unknown_fields)]
pub struct Data {
    pub guid: u32,
    pub name: Option<String>,
    pub name_extra: Option<String>,
    pub female: bool,
    pub race: u8,
    pub class: u8,
    pub level: u8,
    pub locked: bool,
    pub stashed: bool,
    pub online: bool,
    pub metadata: JsonValue,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Form {
    pub role: String,
    pub name: String,
    pub info: Option<String>,
    #[serde(default)] pub name_extra: Option<String>,
    #[serde(default)] pub female: bool,
    pub race: String,
    pub class: String,
    #[serde(default)] pub armor: Option<String>,
    #[serde(default)] pub weapon: Option<String>,
    #[serde(default)] pub traits: HashSet<String>,
    pub location: String,
}

pub struct CreationData {
    pub locked: bool,
    pub name: String,
    pub name_extra: Option<String>,
    pub female: bool,
    pub race: u8,
    pub class: u8,
    pub level: u8,
    pub map: u32,
    pub zone: u32,
    pub position: (f32, f32, f32),
    pub orientation: f32,
    pub banned_spells: HashSet<NonZeroU32>,
    pub innate_spells: HashSet<NonZeroU32>,
    pub starting_skills: HashMap<NonZeroU32, NonZeroU32>,
    pub starting_equip: Option<EquipArray>,
    pub starting_items: HashMap<NonZeroU32, NonZeroU32>,
    pub money: u32,
    pub metadata: JsonValue,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    #[serde(default)] pub info: Option<String>,
    #[serde(default)] pub role: Option<String>,
    #[serde(default)] pub race: Option<String>,
    #[serde(default)] pub class: Option<String>,
    #[serde(default)] pub armor: Option<String>,
    #[serde(default)] pub weapon: Option<String>,
    #[serde(default)] pub traits: Option<HashSet<String>>,
    #[serde(default)] pub location: Option<String>,
}

impl Form {
    pub fn into_cdata(self, campaign: &Campaign) -> AppResult<CreationData> {
        fn ensure(reason: &'static str, success: bool) -> AppResult<()> {
            if success {
                Ok(())
            } else {
                Err(AppError::InvalidInput(reason))
            }
        }

        let name = util::prepare_name(&self.name);
        let name_extra = util::prepare_name_extra(self.name_extra.as_deref());

        ensure("name", NAME_REGEX.is_match(&name))?;
        if let Some(s) = &name_extra { ensure("name_extra", NAME_EXTRA_REGEX.is_match(s))? }

        // fetch entity definitions
        let role = campaign
            .roles
            .get(&self.role)
            .ok_or(AppError::InvalidInput("role"))?;
        let location = campaign
            .system
            .location
            .get(&self.location)
            .ok_or(AppError::InvalidInput("location"))?;
        let race = campaign
            .system
            .race
            .get(&self.race)
            .ok_or(AppError::InvalidInput("race"))?;
        let class = campaign
            .system
            .class
            .get(&self.class)
            .ok_or(AppError::InvalidInput("class"))?;

        let armor = if let Some(id) = &self.armor {
            Some(campaign.system.armor.get(id).ok_or(AppError::InvalidInput("armor"))?)
        } else {
            None
        };
        let weapon = if let Some(id) = &self.weapon {
            Some(campaign.system.weapon.get(id).ok_or(AppError::InvalidInput("weapon"))?)
        } else {
            None
        };
        let traits = fall_iter(self.traits.iter().map(|id| {
            campaign.system.traits.get(id).ok_or(AppError::InvalidInput("traits"))
        })).collect::<Vec<_>>()?;

        // merging all tags
        let mut tags = Tags::new();
        tags.add(
            if self.female {
                "gender/female"
            } else {
                "gender/male"
            },
            1,
        );
        tags.merge_in(&role.provides);
        tags.merge_in(&race.meta.provides);
        tags.merge_in(&class.meta.provides);
        if let Some(a) = &armor {
            tags.merge_in(&a.meta.provides)
        }
        if let Some(w) = &weapon {
            tags.merge_in(&w.meta.provides)
        }
        for t in &traits {
            tags.merge_in(&t.meta.provides)
        }
        tags.merge_in(&location.meta.provides);

        // checking all conditions
        ensure("location/condition", location.meta.requires.as_ref().map(|r| r.check(&tags)).unwrap_or(true))?;
        ensure("race/condition", race.meta.requires.as_ref().map(|r| r.check(&tags)).unwrap_or(true))?;
        ensure("class/condition", class.meta.requires.as_ref().map(|r| r.check(&tags)).unwrap_or(true))?;

        if let Some(a) = &armor {
            ensure("armor/condition", a.meta.requires.as_ref().map(|r| r.check(&tags)).unwrap_or(true))?;
        }
        if let Some(w) = &weapon {
            ensure("weapon/condition", w.meta.requires.as_ref().map(|r| r.check(&tags)).unwrap_or(true))?;
        }
        for t in &traits {
            ensure("traits/condition", t.meta.requires.as_ref().map(|r| r.check(&tags)).unwrap_or(true))?;
        }

        // merging all mods
        let mut mods = Mods::new();

        mods.merge_in(&role.mods);
        mods.merge_in(&location.mods);
        mods.merge_in(&race.mods);
        mods.merge_in(&class.mods);

        if let Some(a) = &armor {
            mods.merge_in(&a.mods)
        }
        if let Some(w) = &weapon {
            mods.merge_in(&w.mods)
        }
        for t in &traits {
            mods.merge_in(&t.mods)
        }

        Ok(CreationData {
            locked: role.kind != RoleKind::Free,
            name,
            name_extra,
            female: self.female,
            race: race.game_id,
            class: class.game_id,
            level: min(LEVEL_MAX, 1 + max(LEVEL_MIN, mods.level)) as u8,
            map: location.map,
            zone: location.zone,
            position: location.position,
            orientation: location.orientation,
            banned_spells: mods.spells_banned,
            innate_spells: mods.spells,
            starting_skills: make_pairs_map(&mods.skills),
            starting_equip: Some(make_equip_array(armor, weapon)),
            starting_items: make_pairs_map(&mods.items),
            money: max(0, mods.money) as u32,
            metadata: json!({
                "info": self.info,
                "role": self.role,
                "race": self.race,
                "class": self.class,
                "armor": self.armor,
                "weapon": self.weapon,
                "traits": self.traits,
                "location": self.location,
            }),
        })
    }
}

pub async fn create(db: MySqlPool, account: u32, data: CreationData) -> AppResult<u32> {
    let done = sqlx::query!(
        "INSERT INTO characters (\
         account, \
         locked, \
         at_login, \
         name, \
         nameExtra, \
         gender, \
         race, \
         class, \
         level, \
         map, \
         zone, \
         position_x, \
         position_y, \
         position_z, \
         orientation, \
         bannedSpells, \
         innateSpells, \
         startingSkills, \
         startingEquip, \
         startingItems, \
         money, \
         metadata) \
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        account,
        if data.locked { 1 } else { 0 },
        (AtLoginFlags::FIRST_LOGIN | AtLoginFlags::CUSTOMIZE).bits,
        data.name,
        data.name_extra,
        if data.female { 1 } else { 0 },
        data.race,
        data.class,
        data.level,
        data.map,
        data.zone,
        data.position.0,
        data.position.1,
        data.position.2,
        data.orientation,
        make_ids_string(data.banned_spells.into_iter()),
        make_ids_string(data.innate_spells.into_iter()),
        make_pairs_string(data.starting_skills.into_iter()),
        data.starting_equip.map(|e| make_equip_string(&e)),
        make_pairs_string(data.starting_items.into_iter()),
        data.money,
        serde_json::to_value(data.metadata).map_err(anyhow::Error::from)?)
        .execute(&db)
        .await?;
    Ok(done.last_insert_id() as u32)
}

pub async fn read(db: MySqlPool, guid: u32) -> AppResult<Data> {
    sqlx::query!(
        "SELECT \
         guid, \
         name, \
         nameExtra AS name_extra, \
         gender, \
         race, \
         class, \
         level, \
         locked, \
         stashed, \
         online, \
         metadata \
         FROM characters \
         WHERE guid = ?",
        guid)
        .fetch_one(&db)
        .await
        .map(|d| Data {
            guid: d.guid,
            name: d.name,
            name_extra: d.name_extra,
            female: d.gender != 0,
            race: d.race,
            class: d.class,
            level: d.level,
            locked: d.locked != 0,
            stashed: d.stashed != 0,
            online: d.online != 0,
            metadata: d.metadata.unwrap_or(JsonValue::Null),
        })
        .map_err(From::from)
}

pub async fn list_mine(db: MySqlPool, account: u32) -> AppResult<Vec<Data>> {
    sqlx::query!(
        "SELECT \
         guid, \
         name, \
         nameExtra AS name_extra, \
         gender, \
         race, \
         class, \
         level, \
         locked, \
         stashed, \
         online, \
         metadata \
         FROM characters \
         WHERE account = ? AND name IS NOT NULL \
         ORDER BY guid DESC",
        account)
        .fetch_all(&db)
        .await
        .map(|v| {
            v.into_iter()
                .map(|d| Data {
                    guid: d.guid,
                    name: d.name,
                    name_extra: d.name_extra,
                    female: d.gender != 0,
                    race: d.race,
                    class: d.class,
                    level: d.level,
                    locked: d.locked != 0,
                    stashed: d.stashed != 0,
                    online: d.online != 0,
                    metadata: d.metadata.unwrap_or(JsonValue::Null),
                })
                .collect()
        })
        .map_err(From::from)
}
pub async fn list_other(db: MySqlPool, account: u32) -> AppResult<Vec<Data>> {
    sqlx::query!(
        "SELECT \
         guid, \
         name, \
         nameExtra AS name_extra, \
         gender, \
         race, \
         class, \
         level, \
         locked, \
         stashed, \
         online, \
         metadata \
         FROM characters \
         WHERE account <> ? AND name IS NOT NULL \
         ORDER BY guid DESC",
        account)
        .fetch_all(&db)
        .await
        .map(|v| {
            v.into_iter()
                .map(|d| Data {
                    guid: d.guid,
                    name: d.name,
                    name_extra: d.name_extra,
                    female: d.gender != 0,
                    race: d.race,
                    class: d.class,
                    level: d.level,
                    locked: d.locked != 0,
                    stashed: d.stashed != 0,
                    online: d.online != 0,
                    metadata: d.metadata.unwrap_or(JsonValue::Null),
                })
                .collect()
        })
        .map_err(From::from)
}

pub async fn check_name(db: MySqlPool, name: &str) -> AppResult<bool> {
    sqlx::query!("SELECT COUNT(*) AS num FROM characters WHERE name = ?", util::prepare_name(name))
        .fetch_one(&db)
        .await
        .map(|r| r.num == 0)
        .map_err(From::from)
}

fn make_pairs_map(src: &HashMap<NonZeroU32, i32>) -> HashMap<NonZeroU32, NonZeroU32> {
    src.into_iter()
        .flat_map(|(id, value)| {
            NonZeroU32::new(max(0, *value) as u32).map(|nonzero| (*id, nonzero))
        })
        .collect()
}

fn make_equip_array(armor: Option<&Armor>, weapon: Option<&Weapon>) -> EquipArray {
    [
        armor.and_then(|e| e.head),
        armor.and_then(|e| e.neck),
        armor.and_then(|e| e.shoulders),
        armor.and_then(|e| e.body),
        armor.and_then(|e| e.chest),
        armor.and_then(|e| e.waist),
        armor.and_then(|e| e.legs),
        armor.and_then(|e| e.feet),
        armor.and_then(|e| e.wrists),
        armor.and_then(|e| e.hands),
        armor.and_then(|e| e.fingers.iter().nth(0).cloned()),
        armor.and_then(|e| e.fingers.iter().nth(1).cloned()),
        armor.and_then(|e| e.trinkets.iter().nth(0).cloned()),
        armor.and_then(|e| e.trinkets.iter().nth(1).cloned()),
        armor.and_then(|e| e.back),
        weapon.and_then(|e| e.mainhand),
        weapon.and_then(|e| e.offhand),
        weapon.and_then(|e| e.ranged),
        armor.and_then(|e| e.tabard),
        armor.and_then(|e| e.bags.iter().nth(0).cloned()),
        armor.and_then(|e| e.bags.iter().nth(1).cloned()),
        armor.and_then(|e| e.bags.iter().nth(2).cloned()),
        armor.and_then(|e| e.bags.iter().nth(3).cloned()),
    ]
}

struct StringBuilder {
    output: String,
    element_index: usize,
}

impl StringBuilder {
    fn new() -> Self {
        Self {
            output: String::new(),
            element_index: 0,
        }
    }

    fn write(&mut self, value: u32) {
        use std::fmt::Write;
        if self.element_index != 0 {
            self.output.push(' ')
        }
        write!(&mut self.output, "{}", value).unwrap();
        self.element_index += 1;
    }

    fn result(self) -> String {
        self.output
    }

    fn result_option(self) -> Option<String> {
        if self.element_index != 0 {
            Some(self.output)
        } else {
            None
        }
    }
}

fn make_equip_string(input: &EquipArray) -> String {
    let mut b = StringBuilder::new();
    for maybe_id in input {
        if let Some(id) = maybe_id {
            b.write(id.get());
        } else {
            b.write(0);
        }
    }
    b.result()
}

fn make_ids_string(input: impl IntoIterator<Item = NonZeroU32>) -> Option<String> {
    let mut b = StringBuilder::new();
    for id in input {
        b.write(id.get());
    }
    b.result_option()
}

fn make_pairs_string(input: impl IntoIterator<Item = (NonZeroU32, NonZeroU32)>) -> Option<String> {
    let mut b = StringBuilder::new();
    for (id, value) in input {
        b.write(id.get());
        b.write(value.get());
    }
    b.result_option()
}
