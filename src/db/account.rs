use sqlx::FromRow;
use sqlx::postgres::PgPool;
use crate::error::{AppError, AppResult};
use crate::util;

const MAX_FAILED_LOGINS: i32 = 2;

#[derive(Debug, Clone, FromRow)]
pub struct Account {
    pub account_id: i32,
    pub nick: String,
    pub email: String,
    pub access_level: i16,
    pub created_at: chrono::NaiveDateTime,
}

pub async fn create(db: PgPool, email: &str, nick: &str, password: &str) -> AppResult<Account> {
    sqlx::query_as!(
        Account,
        "INSERT INTO account (email, nick, password_hash) VALUES ($1, $2, $3) \
         RETURNING account_id, nick, email, access_level, created_at",
        email,
        nick,
        make_password_hash(email, password))
        .fetch_one(&db)
        .await
        .map_err(From::from)
}

pub async fn read(db: PgPool, account_id: i32) -> AppResult<Account> {
    sqlx::query_as!(
        Account,
        "SELECT account_id, nick, email, access_level, created_at \
         FROM account WHERE account_id = $1 LIMIT 1",
        account_id)
        .fetch_one(&db)
        .await
        .map_err(From::from)
}

pub async fn update_nick(db: PgPool, account_id: i32, nick: &str) -> AppResult<()> {
    sqlx::query!(
        "UPDATE account SET nick = $2 WHERE account_id = $1",
        account_id,
        nick)
        .execute(&db)
        .await?;
    Ok(())
}

pub async fn update_email_password(db: PgPool, account_id: i32, email: &str, password: &str) -> AppResult<()> {
    sqlx::query!(
        "UPDATE account SET email = $2, password_hash = $3 WHERE account_id = $1",
        account_id,
        email,
        make_password_hash(email, password))
        .execute(&db)
        .await
        .map(|_| ())
        .map_err(From::from)
}

pub async fn delete(db: PgPool, account_id: i32) -> AppResult<()> {
    sqlx::query!("DELETE FROM account WHERE account_id = $1", account_id)
        .execute(&db)
        .await
        .map(|_| ())
        .map_err(From::from)
}

#[derive(Debug)]
pub enum LoginOutcome {
    NotFound,
    CaptchaRequired,
    WrongPassword,
    Success(Account),
}

pub async fn login(db: PgPool, email_or_nick: &str, password: &str, captcha_validated: bool) -> AppResult<LoginOutcome> {
    let maybe_account = sqlx::query!(
        "SELECT account_id, nick, email, access_level, failed_logins, created_at, password_hash \
         FROM account \
         WHERE lower(email) = lower($1) OR lower(nick) = lower($1) \
         LIMIT 1",
        email_or_nick)
        .fetch_optional(&db)
        .await?;

    let account = if let Some(account) = maybe_account {
        account
    } else {
        return Ok(LoginOutcome::NotFound)
    };

    if account.failed_logins >= MAX_FAILED_LOGINS && !captcha_validated {
        return Ok(LoginOutcome::CaptchaRequired);
    }

    if account.password_hash != make_password_hash(&account.email, password) {
        incr_failed_logins(db.clone(), account.account_id).await;
        return Ok(LoginOutcome::WrongPassword);
    }

    reset_failed_logins(db.clone(), account.account_id).await;
    Ok(LoginOutcome::Success(Account {
        account_id: account.account_id,
        email: account.email,
        nick: account.nick,
        access_level: account.access_level,
        created_at: account.created_at,
    }))
}

async fn incr_failed_logins(db: PgPool, account_id: i32) {
    sqlx::query!("UPDATE account SET failed_logins = failed_logins + 1 WHERE account_id = $1", account_id)
        .execute(&db)
        .await;
}

async fn reset_failed_logins(db: PgPool, account_id: i32) {
    sqlx::query!("UPDATE account SET failed_logins = 0 WHERE account_id = $1", account_id)
        .execute(&db)
        .await;
}

fn make_password_hash(email: &str, password: &str) -> String {
    let input = format!("{}:{}", email.to_uppercase(), password.to_uppercase());
    let digest = ring::digest::digest(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, input.as_bytes());
    util::hexstring(digest)
}
