use sqlx::FromRow;
use sqlx::postgres::PgPool;
use super::DBResult;
use crate::util;

#[derive(Debug, Clone, FromRow)]
pub struct Account {
    pub account_id: i32,
    pub nick: String,
    pub email: String,
    pub access_level: i16,
    pub created_at: chrono::NaiveDateTime,
}

pub async fn create(db: PgPool, email: &str, nick: &str, password: &str) -> DBResult<i32> {
    let password_hash = util::make_password_hash(password.as_bytes())?;

    let result = sqlx::query!(
        "INSERT INTO accounts (email, nick, password_hash) VALUES ($1, $2, $3) RETURNING account_id",
        email,
        nick,
        password_hash)
        .fetch_one(&db)
        .await?;
    Ok(result.account_id)
}

pub async fn read(db: PgPool, account_id: i32) -> DBResult<Account> {
    sqlx::query_as!(
        Account,
        "SELECT account_id, nick, email, access_level, created_at FROM accounts WHERE account_id = $1 LIMIT 1",
        account_id)
        .fetch_one(&db)
        .await
        .map_err(From::from)
}

pub async fn update_email(db: PgPool, account_id: i32, email: &str) -> DBResult<()> {
    sqlx::query!(
        "UPDATE accounts SET email = $2 WHERE account_id = $1",
        account_id,
        email)
        .execute(&db)
        .await?;
    Ok(())
}

pub async fn update_nick(db: PgPool, account_id: i32, nick: &str) -> DBResult<()> {
    sqlx::query!(
        "UPDATE accounts SET nick = $2 WHERE account_id = $1",
        account_id,
        nick)
        .execute(&db)
        .await?;
    Ok(())
}

pub async fn update_password(db: PgPool, account_id: i32, password: &str) -> DBResult<()> {
    let password_hash = util::make_password_hash(password.as_bytes())?;
    sqlx::query!(
        "UPDATE accounts SET password_hash = $2 WHERE account_id = $1",
        account_id,
        password_hash)
        .execute(&db)
        .await?;
    Ok(())
}

pub async fn delete(db: PgPool, account_id: i32) -> DBResult<()> {
    sqlx::query!(
        "DELETE FROM accounts WHERE account_id = $1",
        account_id)
        .execute(&db)
        .await?;
    Ok(())
}

pub async fn login(db: PgPool, email_or_nick: &str, password: &str) -> DBResult<Option<Account>> {
    let a = sqlx::query!(
        "SELECT account_id, nick, email, access_level, created_at, password_hash FROM accounts WHERE email = lower($1) OR nick = lower($1) LIMIT 1",
        email_or_nick)
        .fetch_one(&db)
        .await?;

    if argon2::verify_encoded(&a.password_hash, password.as_bytes())? {
        Ok(Some(Account {
            account_id: a.account_id,
            email: a.email,
            nick: a.nick,
            access_level: a.access_level,
            created_at: a.created_at,
        }))
    } else {
        Ok(None)
    }
}
