use serde::{Serialize, Deserialize};
use mobc_redis::redis;
use crate::util;
use crate::errors::TerraResult;
use super::RedisConn;

#[derive(Serialize, Deserialize)]
pub struct SessionData {
    pub account_id: u32,
}

fn redis_session_key(key: &str) -> String {
    format!("session/{}", key)
}

pub async fn create_session_data(conn: &mut RedisConn, session_data: &SessionData) -> TerraResult<String> {
    let session_key = util::generate_session_key()?;
    let session_data_json = serde_json::to_string(&session_data).map_err(anyhow::Error::from)?;

    redis::cmd("SET")
        .arg(redis_session_key(&session_key))
        .arg(session_data_json)
        .query_async(conn as &mut mobc_redis::Connection)
        .await?;

    Ok(session_key)
}

pub async fn fetch_session_data(conn: &mut RedisConn, session_key: &str) -> TerraResult<Option<SessionData>> {
    let session_data_json: Option<String> =
        redis::cmd("GET")
        .arg(redis_session_key(session_key))
        .query_async(conn as &mut mobc_redis::Connection)
        .await?;

    if let Some(json) = session_data_json {
        let session_data: SessionData = serde_json::from_str(&json)?;
        Ok(Some(session_data))
    } else {
        Ok(None)
    }
}

pub async fn delete_session_data(conn: &mut RedisConn, session_key: &str) -> TerraResult<()> {
    redis::cmd("DEL")
        .arg(redis_session_key(session_key))
        .query_async::<_, ()>(conn as &mut mobc_redis::Connection)
        .await
        .map_err(|e| e.into())
}
