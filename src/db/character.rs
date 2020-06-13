use sqlx::FromRow;
use sqlx::postgres::PgPool;
use super::DBResult;
use crate::util;
use crate::framework::campaign::Campaign;

pub struct AccountRef {
    pub account_id: i32,
    pub nick: String,
    pub access_level: i16,
}

pub enum Gender {
    Male,
    Female,
}

pub enum Status {
    Pending,
    Reviewed,
    Rejected,
    Finalized,
}

pub struct Data {
    pub role: Option<String>,
    pub gender: Gender,
    pub race: u8,
    pub class: u8,
    pub armor: Option<String>,
    pub weapon: Option<String>,
    pub traits: HashSet<String>,
    pub location: String,
    pub name: String,
    pub name_extra: Option<String>,
    pub info_public: String,
    pub info_hidden: String,
    pub private: bool,
}

pub struct PartialData {
    pub name: String,
    pub name_extra: Option<String>,
    pub info_public: String,
    pub info_hidden: String,
    pub private: bool,
}

impl Data {
    pub fn validate(&self, campaign: &Campaign) -> bool {
        let role = campaign.roles.get(&self.role).ok_or(false)?;
        let race = campaign.system.race.get(self.race).ok_or(false)?;
        let class = campaign.system.class.get(self.class).ok_or(false)?;
        let armor = self.armor.map(|id| campaign.system.armor.get(id).unwrap());
        let weapon = self.weapon.map(|id| campaign.system.weapon.get(id).unwrap());
        let traits = self.traits.iter().map(|id| campaign.system.traits.get(id).unwrap());
        let location = campaign.system.location.get(self.location).ok_or(false)?;

        use std::collections::HashSet;
        let tags = HashSet::new();
        tags.extend(&role.provides);
        tags.extend(&race.info.provides);
        tags.extend(&class.info.provides);
        tags.extend(&armor.info.provides);
        tags.extend(&weapon.info.provides);
        for t in traits { tags.extend(&t.info.provides) }
        tags.extend(&location.info.provides);

        role.racemask.contains(self.race) &&
            role.classmask.contains(self.class) &&
            race.info.requires.check(&tags) &&
            class.info.requires.check(&tags) &&
            armor.info.requires.check(&tags) &&
            weapon.info.requires.check(&tags) &&
            traits.iter().all(|t| t.info.requires.check(&tags)) &&
            location.info.requires.check(&tags);
    }
}

pub struct Record {
    pub character_id: i32,
    pub account: Option<AccountRef>,
    pub status: Status,
    pub campaign: String,
    pub data: Data,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

pub struct Summary {
    pub character_id: i32,
    pub name: String,
    pub name_extra: Option<String>,
    pub gender: Gender,
    pub race: i16,
    pub class: i16,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

pub async fn create(db: PgPool, account_id: i32, campaign: &Campaign, data: Data) -> DBResult<i32> {
    let result = sqlx::query!(
        "INSERT INTO characters (\
         account_id, \
         campaign, \
         role, \
         gender, \
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
         RETURNING character_id",
        account_id,
        &campaign.id,
        data.role,
        data.gender,
        data.race,
        data.class,
        data.armor,
        data.weapon,
        data.traits,
        data.location,
        data.name,
        data.name_extra,
        data.info_public,
        data.info_hidden,
        data.private)
        .fetch_one(&db)
        .await?;
    Ok(result.character_id)
}

pub async fn read(db: PgPool, character_id: i32) -> DBResult<Record> {
    todo!()
}

pub async fn update_full(db: PgPool, character_id: i32, data: Data) -> DBResult<()> {
    sqlx::query!(
        "UPDATE characters SET \
         role = $1, \
         gender = $2, \
         race = $3, \
         class = $4, \
         armor = $5, \
         weapon = $6, \
         traits = $7, \
         location = $8, \
         name = $9, \
         name_extra = $10, \
         info_public = $11, \
         info_hidden = $12, \
         private = $13 \
         WHERE character_id = $14",
        data.role,
        data.gender,
        data.race,
        data.class,
        data.armor,
        data.weapon,
        data.traits,
        data.location,
        data.name,
        data.name_extra,
        data.info_public,
        data.info_hidden,
        data.private,
        character_id)
        .execute(&db)
        .await?;
    Ok(())
}

pub async fn update_partial(db: PgPool, character_id: i32, data: PartialData) -> DBResult<()> {
    sqlx::query!(
        "UPDATE characters SET \
         name = $1, \
         name_extra = $2, \
         info_public = $3, \
         info_hidden = $4, \
         private = $5 \
         WHERE character_id = $6",
        data.name,
        data.name_extra,
        data.info_public,
        data.info_hidden,
        data.private,
        character_id)
        .execute(&db)
        .await?;
    Ok(())
}

pub async fn update_status(db: PgPool, character_id: i32, status: Option<Status>) -> DBResult<()> {
    todo!()
}

pub async fn update_account_id(db: PgPool, character_id: i32, account_id: i32) -> DBResult<()> {
    todo!()
}

pub async fn list(db: PgPool) -> DBResult<Vec<Summary>> {
    todo!()
}
