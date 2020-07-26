use serde::Serialize;
use sqlx::{mysql::MySqlPool, prelude::*};
use crate::{error::AppResult, util};

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Account {
    pub id: u32,
    pub username: String,
    pub gmlevel: Option<u8>,
}

pub async fn create(db: MySqlPool, username: &str, password: &str) -> AppResult<u32> {
    let done = sqlx::query!(
        "INSERT INTO account (username, sha_pass_hash) VALUES (?,?)",
        username.to_uppercase(),
        make_password_hash(username, password))
        .execute(&db)
        .await?;
    Ok(done.last_insert_id() as u32)
}

pub async fn replace(db: MySqlPool, id: u32, username: &str, password: &str) -> AppResult<()> {
    sqlx::query!(
        "REPLACE INTO account (id, username, sha_pass_hash) VALUES (?,?,?)",
        id,
        username.to_uppercase(),
        make_password_hash(username, password))
        .execute(&db)
        .await
        .map(|_| ())
        .map_err(From::from)
}

pub async fn read(db: MySqlPool, id: u32) -> AppResult<Account> {
    sqlx::query_as!(
        Account,
        "SELECT id, username, gmlevel \
         FROM account \
         LEFT JOIN account_access USING (id) \
         WHERE id = ? \
         LIMIT 1",
        id)
        .fetch_one(&db)
        .await
        .map_err(From::from)
}

pub async fn update(db: MySqlPool, id: u32, username: &str, password: &str) -> AppResult<()> {
    sqlx::query!(
        "UPDATE account SET username = ?, sha_pass_hash = ? WHERE id = ?",
        id,
        username.to_uppercase(),
        make_password_hash(username, password))
        .execute(&db)
        .await
        .map(|_| ())
        .map_err(From::from)
}

// pub async fn delete(db: MySqlPool, id: u32) -> AppResult<()> {
//     let mut tx = db.begin().await?;
//     sqlx::query!("DELETE FROM account WHERE id = ?", id).execute(&mut tx).await?;
//     tx.commit().await?;
//     Ok(())
// }

fn make_password_hash(username: &str, password: &str) -> String {
    let input = format!("{}:{}", username, password).to_uppercase();
    let digest = ring::digest::digest(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, input.as_bytes());
    util::hexstring(digest)
}
