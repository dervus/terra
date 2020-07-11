use std::collections::HashSet;
use sqlx::{FromRow, Row, Type, Postgres};
use sqlx::postgres::PgPool;
use crate::error::{AppError, AppResult};
use crate::util;
use crate::framework::campaign::Campaign;

#[derive(Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(rename = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum CharacterStatus {
    Pending,
    Reviewed,
    Rejected,
    Finalized,
}

#[derive(FromRow)]
pub struct Data1 {
    pub role: String,
    pub female: bool,
    pub race: i16,
    pub class: i16,
    pub armor: String,
    pub weapon: String,
    pub traits: Vec<String>,
    pub location: String,
}

#[derive(FromRow)]
pub struct Data2 {
    pub name: String,
    pub name_extra: Option<String>,
    pub info_public: String,
    pub info_hidden: String,
    pub private: bool,
}

#[derive(FromRow)]
pub struct Character {
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub pc_id: i32,
    pub account_id: i32,
    pub campaign: String,
    pub status: String,
    // pub data1: Data1,
    pub role: String,
    pub female: bool,
    pub race: i16,
    pub class: i16,
    pub armor: String,
    pub weapon: String,
    pub traits: Vec<String>,
    pub location: String,
    // pub data2: Data2,
    pub name: String,
    pub name_extra: Option<String>,
    pub info_public: String,
    pub info_hidden: String,
    pub private: bool,
}

pub fn from_formdata(data: &[(String, String)]) -> AppResult<(Data1, Data2)> {
    let opt = |key: &str| -> Option<String> {
        data.iter().find(|(k,_)| k == key).map(|(_,v)| v.clone())
    };
    let req = |key: &str| -> AppResult<String> {
        opt(key).ok_or(AppError::BadRequest)
    };
    let req_int = |key: &str| -> AppResult<i16> {
        req(key).and_then(|val| val.parse().map_err(|_| AppError::BadRequest))
    };

    let role = req("role")?;
    let female = match req("gender")?.as_ref() {
        "male" => false,
        "female" => true,
        _ => return Err(AppError::BadRequest),
    };
    let race = req_int("race")?;
    let class = req_int("class")?;
    let armor = req("armor")?;
    let weapon = req("weapon")?;
    let traits = data.iter().filter(|(k,_)| k == "traits").map(|(_,v)| v.clone()).collect::<Vec<_>>();
    let location = req("location")?;
    let name = req("name")?;
    let name_extra = opt("name_extra");
    let info_public = req("info_public")?;
    let info_hidden = req("info_hidden")?;
    let private = data.iter().find(|(k,_)| k == "private").is_some();

    let name = util::prepare_name(&name);
    let name_extra = util::prepare_name_extra(name_extra.as_deref());

    Ok(
        (
            Data1 {
                role,
                female,
                race,
                class,
                armor,
                weapon,
                traits,
                location,
            },
            Data2 {
                name,
                name_extra,
                info_public,
                info_hidden,
                private,
            }
        )
    )
}

#[derive(FromRow)]
pub struct CharacterSummary {
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub pc_id: i32,
    pub status: String,
    pub name: String,
    pub name_extra: Option<String>,
    pub female: bool,
    pub race: i16,
    pub class: i16,
}

// impl<'c, R> sqlx::FromRow<'c, R> for Character
// where
//     R: sqlx::Row<'c>,
//     &'c str: sqlx::row::ColumnIndex<'c, R>,
//     chrono::NaiveDateTime: sqlx::decode::Decode<'c, R::Database> + sqlx::types::Type<R::Database>,
//     Option<chrono::NaiveDateTime>: sqlx::decode::Decode<'c, R::Database> + sqlx::types::Type<R::Database>,
//     i32: sqlx::decode::Decode<'c, R::Database> + sqlx::types::Type<R::Database>,
//     String: sqlx::decode::Decode<'c, R::Database> + sqlx::types::Type<R::Database>,
//     CharacterStatus: sqlx::decode::Decode<'c, R::Database> + sqlx::types::Type<R::Database>,
//     Data1: FromRow<'c, R>,
//     Data2: FromRow<'c, R>,
// {
//     fn from_row(row: &R) -> sqlx::Result<Self> {
//         let created_at = row.try_get("created_at")?;
//         let updated_at = row.try_get("updated_at")?;
//         let pc_id = row.try_get("pc_id")?;
//         let account_id = row.try_get("account_id")?;
//         let campaign = row.try_get("campaign")?;
//         let status = row.try_get("status")?;
//         let data1 = Data1::from_row(row)?;
//         let data2 = Data2::from_row(row)?;
//         Ok(Self { created_at, updated_at, pc_id, account_id, campaign, status, data1, data2 })
//     }
// }

macro_rules! get {
    ($key:expr, $from:expr) => {
        if let Some(value) = $from.get($key) {
            value
        } else {
            return false;
        }
    }
}

macro_rules! get_opt {
    ($key:expr, $from:expr) => {
        if let Some(key) = $key {
            if let Some(value) = $from.get(key) {
                value
            } else {
                return false;
            }
        } else {
            return false;
        }
    }
}

impl Data1 {
    pub fn validate(&self, campaign: &Campaign) -> bool {
        let role = get!(&self.role, campaign.roles);
        let race = get!(&(self.race as u8), campaign.system.race);
        let class = get!(&(self.class as u8), campaign.system.class);
        let armor = get!(&self.armor, campaign.system.armor);
        let weapon = get!(&self.weapon, campaign.system.weapon);
        let mut traits = Vec::with_capacity(self.traits.len());
        for id in &self.traits { traits.push(get!(id, campaign.system.traits)) }
        let location = get!(&self.location, campaign.system.location);

        let mut tags = HashSet::new();
        tags.extend(role.provides.iter().cloned());
        tags.extend(race.info.provides.iter().cloned());
        tags.extend(class.info.provides.iter().cloned());
        tags.extend(armor.info.provides.iter().cloned());
        tags.extend(weapon.info.provides.iter().cloned());
        for t in &traits { tags.extend(t.info.provides.iter().cloned()) }
        tags.extend(location.info.provides.iter().cloned());

        use crate::framework::tags::check;
        check(race.info.requires.as_ref(), &tags) &&
            check(class.info.requires.as_ref(), &tags) &&
            check(armor.info.requires.as_ref(), &tags) &&
            check(weapon.info.requires.as_ref(), &tags) &&
            traits.iter().all(|t| check(t.info.requires.as_ref(), &tags)) &&
            check(location.info.requires.as_ref(), &tags)
    }
}

pub async fn create(db: PgPool, account_id: i32, campaign: &Campaign, data1: Data1, data2: Data2) -> AppResult<Character> {
    if !data1.validate(campaign) { return Err(AppError::BadRequest) }
    let traits = data1.traits.iter().cloned().collect::<Vec<_>>();
    sqlx::query_as!(
        Character,
        "INSERT INTO pc (\
         account_id, \
         campaign, \
         role, \
         female, \
         race, \
         class, \
         armor, \
         weapon, \
         traits, \
         location, \
         name, \
         name_extra, \
         info_public, \
         info_hidden, \
         private) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) \
         RETURNING *",
        account_id,
        &campaign.id,
        data1.role,
        data1.female,
        data1.race as i16,
        data1.class as i16,
        data1.armor,
        data1.weapon,
        &traits,
        data1.location,
        data2.name,
        data2.name_extra,
        data2.info_public,
        data2.info_hidden,
        data2.private)
        .fetch_one(&db)
        .await
        .map_err(From::from)
}

pub async fn read(db: PgPool, pc_id: i32) -> AppResult<Character> {
    sqlx::query_as!(
        Character,
        "SELECT * FROM pc WHERE pc_id = $1",
        pc_id)
        .fetch_one(&db)
        .await
        .map_err(From::from)
}

pub async fn update_data1(db: PgPool, pc_id: i32, data1: Data1) -> AppResult<()> {
    sqlx::query!(
        "UPDATE pc SET \
         role = $1, \
         female = $2, \
         race = $3, \
         class = $4, \
         armor = $5, \
         weapon = $6, \
         traits = $7, \
         location = $8 \
         WHERE pc_id = $9",
        data1.role,
        data1.female,
        data1.race,
        data1.class,
        data1.armor,
        data1.weapon,
        &data1.traits,
        data1.location,
        pc_id)
        .execute(&db)
        .await
        .map(|_| ())
        .map_err(From::from)
}

pub async fn update_data2(db: PgPool, pc_id: i32, data2: Data2) -> AppResult<()> {
    sqlx::query!(
        "UPDATE pc SET \
         name = $1, \
         name_extra = $2, \
         info_public = $3, \
         info_hidden = $4, \
         private = $5 \
         WHERE pc_id = $6",
        data2.name,
        data2.name_extra,
        data2.info_public,
        data2.info_hidden,
        data2.private,
        pc_id)
        .execute(&db)
        .await
        .map(|_| ())
        .map_err(From::from)
}

pub async fn update_status(db: PgPool, pc_id: i32, status: String) -> AppResult<()> {
    sqlx::query!("UPDATE pc SET status = $1 WHERE pc_id = $2", status, pc_id)
        .execute(&db)
        .await
        .map(|_| ())
        .map_err(From::from)
}

pub async fn update_account_id(db: PgPool, pc_id: i32, account_id: i32) -> AppResult<()> {
    sqlx::query!("UPDATE pc SET account_id = $1 WHERE pc_id = $2", account_id, pc_id)
        .execute(&db)
        .await
        .map(|_| ())
        .map_err(From::from)
}

pub async fn list(db: PgPool) -> AppResult<Vec<CharacterSummary>> {
    sqlx::query_as!(
        CharacterSummary,
        "SELECT created_at, updated_at, pc_id, status, name, name_extra, female, race, class \
         FROM pc \
         ORDER BY pc_id")
        .fetch_all(&db)
        .await
        .map_err(From::from)
}
