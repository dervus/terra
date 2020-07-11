use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use std::net::SocketAddr;
use log::{info, warn};
use serde::Deserialize;
use sqlx::postgres::PgPool;
use sqlx::mysql::MySqlPool;
use crate::db::DBConfig;
use crate::framework;
use crate::framework::campaign::Campaign;
use crate::util;

#[derive(Deserialize)]
pub struct AppConfig {
    pub listen: SocketAddr,
    pub secret: String,
    pub data_path: PathBuf,
    pub site_db: DBConfig,
    pub wowauth_db: DBConfig,
    pub campaigns: Vec<CampaignConfig>,
    pub recaptcha: RecaptchaConfig,
}

#[derive(Deserialize)]
pub struct RecaptchaConfig {
    pub sitekey: String,
    pub secret: String,
}

#[derive(Deserialize)]
pub struct CampaignConfig {
    pub id: String,
    pub wowchars_db: DBConfig,
}

#[derive(Deserialize)]
pub struct SiteConfig {
    pub title: String,
    #[serde(default)] pub logo: Option<String>,
    #[serde(default)] pub logo_small: Option<String>,
    #[serde(default)] pub menu: Vec<(String, String)>,
}

pub struct SitePages {
    pub front: String,
}

pub struct AppContext {
    pub secret: ring::hmac::Key,
    pub assets_path: PathBuf,
    pub site_config: SiteConfig,
    pub site_pages: SitePages,
    pub site_db: PgPool,
    pub wowauth_db: MySqlPool,
    pub campaigns: Vec<Arc<CampaignContext>>,
    pub campaigns_by_id: HashMap<String, Arc<CampaignContext>>,
    pub recaptcha: RecaptchaConfig,
    pub http_client: reqwest::Client,
}

pub struct CampaignContext {
    pub data: Campaign,
    pub wowchars_db: MySqlPool,
}

pub type CtxRef = Arc<AppContext>;
pub type CCtxRef = Arc<CampaignContext>;

impl std::ops::Drop for AppContext {
    fn drop(&mut self) {
        info!("Shutting down application context");
    }
}

pub async fn create_context(config: AppConfig) -> anyhow::Result<CtxRef> {
    let secret_rawkey = base64::decode_config(&config.secret, base64::URL_SAFE_NO_PAD).unwrap_or_else(|_| {
        warn!("unable to parse secret as base64 payload; using it as is");
        config.secret.as_bytes().to_owned()
    });
    let secret = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, &secret_rawkey);

    let assets_path = config.data_path.join("assets").canonicalize()?;
    let site_config = util::load_yaml(config.data_path.join("site.yml"))?;
    let front_page = util::load_markdown(config.data_path.join("front_page.md"))?;
    let site_pages = SitePages { front: front_page };
    
    let site_db = config.site_db.create_pool().await?;
    let wowauth_db = config.wowauth_db.create_pool().await?;
    
    let mut campaigns = Vec::with_capacity(config.campaigns.len());
    let mut campaigns_by_id = HashMap::with_capacity(config.campaigns.len());

    for ccfg in &config.campaigns {
        info!("Loading campaign {:?}", &ccfg.id);
        let campaign = framework::load_campaign(&config.data_path, &ccfg.id)?;
        let wowchars_db = ccfg.wowchars_db.create_pool().await?;
        let cctx = Arc::new(CampaignContext {
            data: campaign,
            wowchars_db
        });
        campaigns_by_id.insert(ccfg.id.clone(), cctx.clone());
        campaigns.push(cctx);
    }

    Ok(Arc::new(AppContext {
        secret,
        assets_path,
        site_config,
        site_pages,
        site_db,
        wowauth_db,
        campaigns,
        campaigns_by_id,
        recaptcha: config.recaptcha,
        http_client: reqwest::Client::new(),
    }))
}
