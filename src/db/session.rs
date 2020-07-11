use sqlx::postgres::PgPool;
use crate::error::AppResult;
use super::account::Account;
use crate::util;

const SESSION_KEY_LEN: usize = 32;

pub async fn create(db: PgPool, account_id: i32) -> AppResult<String> {
    let session_key = generate_session_key()?;
    sqlx::query!("INSERT INTO session (session_key, account_id) VALUES ($1, $2)", &session_key, account_id).execute(&db).await?;
    Ok(session_key)
}

pub async fn touch(db: PgPool, session_key: &str) -> AppResult<Option<Account>> {
    let result = sqlx::query_as!(
        Account,
        "UPDATE session s \
         SET last_request_at = now_utc() \
         FROM account a \
         WHERE session_key = $1 AND a.account_id = s.account_id \
         RETURNING a.account_id, a.nick, a.email, a.access_level, a.created_at",
        session_key)
        .fetch_optional(&db)
        .await?;
    Ok(result)
}

pub async fn delete(db: PgPool, session_key: &str) -> AppResult<()> {
    sqlx::query!("DELETE FROM session WHERE session_key = $1", session_key)
        .execute(&db)
        .await
        .map(|_| ())
        .map_err(From::from)
}

pub struct LogoutOutcome;

pub async fn logout(db: PgPool, session_key: &str) -> AppResult<LogoutOutcome> {
    delete(db, session_key).await.map(|_| LogoutOutcome)
}

fn generate_session_key() -> AppResult<String> {
    let mut key = [0u8; SESSION_KEY_LEN];
    util::fill_random_bytes(&mut key)?;
    Ok(base64::encode_config(key, base64::URL_SAFE_NO_PAD))
}
