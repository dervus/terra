#![feature(proc_macro_hygiene, async_closure)]

mod errors;
mod util;
mod system;
mod page;
mod views;
mod handlers;
mod db;

use std::path::PathBuf;
use std::net::SocketAddr;
use std::sync::Arc;
use log::info;
use url::Url;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub listen: SocketAddr,
    pub files_path: PathBuf,
    pub data_path: PathBuf,
    pub redis: String,
    pub auth_db: String,
    pub chars_db: String,
    pub campaign: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Initializing logger");
    env_logger::init();

    let mut args = std::env::args();
    args.next();
    let config_path = args.next().unwrap_or("config.yml".to_owned());

    info!("Loading config file {}", &config_path);
    let config: Config = serde_yaml::from_str(&std::fs::read_to_string(&config_path)?)?;
    let redis_url = Url::parse(&config.redis)?;

    info!("Loading shared system {:?}", &config.data_path);
    let shared_system = system::load_shared_system(&config.data_path)?;

    info!("Loading campaign {:?}", &config.campaign);
    let campaign = Arc::new(system::load_campaign(&config.data_path, &config.campaign, Some(&shared_system))?);

    info!("Initilizaing database connections");
    let redis_pool = db::RedisPool::new(mobc_redis::RedisConnectionManager::new(mobc_redis::redis::Client::open(redis_url)?));
    let auth_pool = db::MysqlPool::from_url(&config.auth_db)?;
    let chars_pool = db::MysqlPool::from_url(&config.chars_db)?;

    info!("Setting up HTTP server");
    let app = handlers::make_app(handlers::AppConfig {
        files_path: config.files_path,
        data_path: config.data_path,
        redis_pool,
        auth_pool,
        chars_pool,
        campaign,
    });

    warp::serve(app).run(config.listen).await;
    Ok(())
}
