#![feature(proc_macro_hygiene, async_closure)]

mod util;
mod system;
mod render;
mod views;
mod handlers;
mod db;

use std::path::PathBuf;
use std::net::SocketAddr;
use std::sync::Arc;
use warp::Filter;
use render::Page;
use serde::{Serialize, Deserialize};

pub type RedisPool = mobc::Pool<mobc_redis::RedisConnectionManager>;
pub type RedisConn = mobc::Connection<mobc_redis::RedisConnectionManager>;
pub type MysqlPool = mysql_async::Pool;
pub type MysqlConn = mysql_async::Conn;

#[derive(Debug)] pub struct MysqlFailure;
impl warp::reject::Reject for MysqlFailure {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub listen: SocketAddr,
    pub files_path: PathBuf,
    pub data_path: PathBuf,
    pub redis: String,
    pub auth_db: String,
    pub chars_db: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let config: Config = serde_yaml::from_str(&std::fs::read_to_string("config.yml")?)?;
    let redis_url = url::Url::parse(&config.redis)?;
    
    let shared_system = system::load_shared_system(&config.data_path)?;
    let mut campaigns = Vec::new();
    for id in &["last-bastion"] {
        let loaded = system::load_campaign(&config.data_path, id, Some(&shared_system))?;
        campaigns.push(loaded);
    }

    let redis = RedisPool::new(mobc_redis::RedisConnectionManager::new(mobc_redis::redis::Client::open(redis_url)?));
    let auth_db = MysqlPool::from_url(&config.auth_db)?;
    let chars_db = MysqlPool::from_url(&config.chars_db)?;

    let with_redis = warp::any().map(move || redis.clone());
    let with_auth_db = warp::any().map(move || auth_db.clone());
    let with_chars_db = warp::any().map(move || chars_db.clone());

    let session_key = warp::cookie::optional("session_key");
    // let user = session_key.map(|key: Option<String>| key.and_then(|k| k.parse::<u32>().ok())).and_then(move |maybe_id: Option<u32>| {
    //     let pool = auth_db.clone();
    //     async move {
    //         let conn = pool.get_conn().await.map_err(|_| warp::reject::custom(MysqlFailure))?;
    //         if let Some(id) = maybe_id {
    //             db::fetch_account_info(conn, id).await.map_err(|_| warp::reject::custom(MysqlFailure))
    //         } else {
    //             Ok::<Option<db::AccountInfo>, warp::Rejection>(None)
    //         }
    //     }
    // });
    let user_id = session_key.map(|key: Option<String>| key.and_then(|k| k.parse::<u32>().ok()));
    let user = user_id.and(with_auth_db).and_then(async move |maybe_id: Option<u32>, pool: MysqlPool| {
        let conn = pool.get_conn().await.map_err(|_| warp::reject::custom(MysqlFailure))?;
        if let Some(id) = maybe_id {
            db::fetch_account_info(conn, id).await.map_err(|_| warp::reject::custom(MysqlFailure))
        } else {
            Ok::<Option<db::AccountInfo>, warp::Rejection>(None)
        }
    });

    let index = warp::path::end().map(move || warp::reply::html(Page::new().render(views::campaign(&campaigns[0])).into_string()));
    let test = warp::path!("test").and(user).map(|u| format!("Cookie: {:?}", u));
    let set_test = warp::path!("set-test" / String).map(|s| warp::reply::with_header(warp::reply(), "set-cookie", cookie::Cookie::build("session_key", s).path("/").same_site(cookie::SameSite::Strict).finish().to_string()));
    let static_files = warp::path("static").and(warp::fs::dir(config.files_path.clone()));
    let app = warp::get().and(static_files.or(test).or(set_test).or(index)).with(warp::log::log("terra"));
    warp::serve(app).run(config.listen).await;
    
    Ok(())
}
