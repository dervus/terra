use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use log::info;
use serde::Deserialize;
use sqlx::mysql::MySqlPool;
use crate::{db::DBConfig, framework, framework::campaign::Campaign};

#[derive(Deserialize)]
pub struct AppConfig {
    pub listen: SocketAddr,
    pub campaign_path: PathBuf,
    #[serde(default)]
    pub assets_path: Option<PathBuf>,
    pub auth_db: DBConfig,
    pub chars_db: DBConfig,
}

pub struct AppContext {
    pub campaign: Campaign,
    pub auth_db: MySqlPool,
    pub chars_db: MySqlPool,
}

pub type CtxRef = Arc<AppContext>;

pub fn create_context(config: AppConfig) -> anyhow::Result<CtxRef> {
    info!(
        "Initializing auth database connection pool: {}",
        &config.auth_db.connect
    );
    let auth_db = futures::executor::block_on(config.auth_db.create_pool())?;

    info!(
        "Initializing characters database connection pool: {}",
        &config.chars_db.connect
    );
    let chars_db = futures::executor::block_on(config.chars_db.create_pool())?;

    info!("Loading campaign data: {:?}", &config.campaign_path);
    let campaign = framework::load_campaign(config.campaign_path, config.assets_path)?;

    Ok(Arc::new(AppContext {
        campaign,
        auth_db,
        chars_db,
    }))
}
