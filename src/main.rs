#![feature(proc_macro_hygiene, async_closure)]

mod util;
mod db;
mod framework;

use std::path::PathBuf;
use std::net::SocketAddr;
use log::info;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    pub listen: SocketAddr,
    pub data_path: PathBuf,
    pub database_url: String,
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

    info!("Loading campaign {:?}", &config.campaign);
    let campaign = framework::load_campaign(&config.data_path, &config.campaign)?;
    println!("{:#?}", campaign);

    // info!("Initilizaing database connections");
    // let redis_pool = db::RedisPool::new(mobc_redis::RedisConnectionManager::new(mobc_redis::redis::Client::open(redis_url)?));
    // let auth_pool = db::MysqlPool::from_url(&config.auth_db)?;
    // let chars_pool = db::MysqlPool::from_url(&config.chars_db)?;

    // info!("Setting up HTTP server");
    // let app = server::make_app(server::AppConfig {
    //     files_path: config.files_path,
    //     data_path: config.data_path,
    //     redis_pool,
    //     auth_pool,
    //     chars_pool,
    //     campaign,
    // });

    // warp::serve(app).run(config.listen).await;
    Ok(())
}
