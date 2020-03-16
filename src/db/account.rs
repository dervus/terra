use ring::digest::{SHA1_FOR_LEGACY_USE_ONLY, digest};
use mysql_async::prelude::*;
use crate::util;
use super::{MysqlConn, MysqlResult};

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub id: u32,
    pub nick: String,
    pub access_level: u8,
}

pub async fn fetch_account_info(conn: MysqlConn, id: u32) -> MysqlResult<Option<AccountInfo>> {
    const SQL: &'static str = "\
SELECT a.id, a.username, aa.gmlevel \
FROM account a LEFT JOIN account_access aa ON (a.id = aa.id AND aa.RealmID = -1) \
WHERE a.id = ?";

    let (conn, result) = conn.first_exec(SQL, (id,)).await?;
    let fields: Option<(u32, String, Option<u8>)> = result;

    Ok((conn, fields.map(|(id, nick, access_level)| {
        AccountInfo { id, nick, access_level: access_level.unwrap_or(0) }
    })))
}

pub async fn login_query(conn: MysqlConn, login: &str, password: &str) -> MysqlResult<Option<AccountInfo>> {
    const SQL: &'static str = "\
SELECT a.id, a.username, a.sha_pass_hash, aa.gmlevel \
FROM account a LEFT JOIN account_access aa ON (a.id = aa.id AND aa.RealmID = -1) \
WHERE UPPER(a.username) = ? OR UPPER(a.email) = ?";

    let query = login.trim().to_uppercase();
    let (conn, result) = conn.first_exec(SQL, (&query, &query)).await?;
    let fields: Option<(u32, String, String, Option<u8>)> = result;

    Ok((conn, fields.and_then(|(id, nick, actual_passhash, access_level)| {
        let input_raw = format!("{}:{}", &nick, password.trim()).to_uppercase();
        let input_passhash = util::hexstring(digest(&SHA1_FOR_LEGACY_USE_ONLY, input_raw.as_bytes()).as_ref());

        if input_passhash == actual_passhash {
            Some(AccountInfo { id, nick, access_level: access_level.unwrap_or(0) })
        } else {
            None
        }
    })))
}
