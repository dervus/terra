use sqlx::postgres::PgPool;
use super::DBResult;
use crate::util;

const SESSION_KEY_LEN: usize = 32;

pub async fn create(db: PgPool, account_id: i32) -> DBResult<Vec<u8>> {
    let mut session_key = [0u8; SESSION_KEY_LEN];
    util::fill_random_bytes(&mut session_key)?;
    sqlx::query!("INSERT INTO sessions (session_key, account_id) VALUES ($1, $2)", session_key.as_ref(), account_id).execute(&db).await?;
    Ok(session_key.into())
}

pub async fn touch(db: PgPool, session_key: &[u8]) -> DBResult<i32> {
    let result = sqlx::query!("UPDATE sessions SET last_access_at = now_utc() WHERE session_key = $1 RETURNING account_id", session_key).fetch_one(&db).await?;
    Ok(result.account_id)
}

pub fn encode_key(session_key: &[u8]) -> String {
    base64::encode_config(session_key, base64::URL_SAFE_NO_PAD)
}

pub fn decode_key(session_key: &str) -> Result<Vec<u8>, base64::DecodeError> {
    base64::decode_config(session_key, base64::URL_SAFE_NO_PAD)
}
