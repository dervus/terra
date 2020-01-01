use rocket_contrib::databases::mysql;
use anyhow::Result;

#[database("auth")]
pub struct AuthDB(mysql::Conn);

#[database("characters")]
pub struct CharsDB(mysql::Conn);

pub struct AccountInfo {
    pub id: u32,
    pub nick: String,
    pub access_level: u8,
}

impl AccountInfo {
    pub fn href(&self) -> String {
        uri!(crate::handlers::user_page: &self.nick).to_string()
    }
}

pub fn fetch_account_info(conn: &mut mysql::Conn, id: u32) -> Result<Option<AccountInfo>> {
    const SQL: &'static str = "\
SELECT a.id, a.username, aa.gmlevel \
FROM account a LEFT JOIN account_access aa ON (a.id = aa.id AND aa.RealmID = -1) \
WHERE a.id = ?";

    let result: Option<(u32, String, u8)> = conn.first_exec(SQL, (id,))?;
    Ok(result.map(|(id, nick, access_level)| AccountInfo { id, nick, access_level }))
}

pub struct OxyCharCreateInfo {
    pub guid: u32,
    pub account: u32,
    pub name: String,
    pub race: u8,
    pub model_override: Option<u32>,
    pub model_scale: f32,
    pub class: u8,
    pub gender: crate::system::Gender,
    pub level: u8,
    pub max_level: Option<u8>,
    pub money: u32,
    pub position: (f32, f32, f32),
    pub map: u16,
    pub orientation: f32,
    pub at_login: u16,
    pub base_spells: std::collections::HashSet<u32>,
    pub starting_equip: Vec<Option<u32>>,
    pub starting_items: std::collections::HashMap<u32, u8>,
    pub zone: u16,
}

impl OxyCharCreateInfo {
    pub fn to_sql(&self) -> String {
        format!(
            "INSERT INTO characters (guid, account, name, race, modelOverride, modelScale, class, gender, level, maxLevel, money, position_x, position_y, position_z, map, orientation, at_login, baseSpells, startingEquip, startingItems, zone, taximask, cinematic) \
             VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '', '1');",
            self.guid,
            self.account,
            self.name,
            self.race,
            self.model_override.unwrap_or(0),
            self.model_scale,
            self.class,
            match self.gender { crate::system::Gender::Male => 0, crate::system::Gender::Female => 1 },
            self.level,
            self.max_level.unwrap_or(0),
            self.money,
            self.position.0,
            self.position.1,
            self.position.2,
            self.map,
            self.orientation,
            self.at_login,
            self.base_spells.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "),
            self.starting_equip.iter().map(|x| x.unwrap_or(0).to_string()).collect::<Vec<_>>().join(" "),
            self.starting_items.iter().map(|(k,v)| format!("{} {}", k, v)).collect::<Vec<_>>().join(" "),
            self.zone
        )
    }
}

const CHARACTER_COLUMNS: [&'static str; 21] = [
    "guid",
    "account",
    "name",
    "race",
    "modelOverride",
    "modelScale",
    "class",
    "gender",
    "level",
    "maxLevel",
    "money",
    "position_x",
    "position_y",
    "position_z",
    "map",
    "orientation",
    "at_login",
    "baseSpells",
    "startingEquip",
    "startingItems",
    "zone",
];

fn make_insert_stmt(values: &[&str]) -> String {
    use std::fmt::Write;
    let mut out = String::from("INSERT INTO characters (");
    for (index, column) in values.into_iter().enumerate() {
        if index != 0 { out.write_str(", ").unwrap() }
        out.write_str(column).unwrap();
    }
    out.write_str(") VALUES (");
    for index in 0..values.len() {
        if index != 0 { out.write_str(", ").unwrap() }
        out.write_char('?').unwrap();
    }
    out.write_char(')').unwrap();
    out
}

fn make_update_stmt(values: &[&str]) -> String {
    use std::fmt::Write;
    let mut out = String::from("UPDATE characters SET ");
    for (index, column) in values.into_iter().enumerate() {
        if index != 0 { out.write_str(", ").unwrap() }
        write!(&mut out, "{} = ?", column).unwrap();
    }
    out.write_str(" WHERE guid = ?").unwrap();
    out
}
