#![feature(proc_macro_hygiene, async_closure)]

mod util;
mod db;
mod framework;
mod view;
mod web;

use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use std::net::SocketAddr;
use log::info;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    listen: SocketAddr,
    data_path: PathBuf,
    main_db: db::DBConfig,
    campaigns: Vec<CampaignConfig>,
}

#[derive(Deserialize)]
struct CampaignConfig {
    id: String,
    chars_db: db::DBConfig,
}

#[derive(Deserialize)]
struct SiteConfig {
    title: String,
    logo: String,
    logo_small: String,
}

struct SitePages {
    main: String,
}

struct Context {
    data_path: PathBuf,
    site_config: SiteConfig,
    site_pages: SitePages,
    main_db: sqlx::postgres::PgPool,
    campaigns: Vec<Arc<CampaignContext>>,
    campaigns_by_id: HashMap<String, Arc<CampaignContext>>,
}

struct CampaignContext {
    data: framework::campaign::Campaign,
    chars_db: sqlx::mysql::MySqlPool,
}

type CtxRef = Arc<Context>;
type CCtxRef = Arc<CampaignContext>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut args = std::env::args();
    args.next();
    let config_path = args.next().unwrap_or("config.yml".to_owned());

    info!("Loading config file {}", &config_path);
    let config: Config = serde_yaml::from_str(&std::fs::read_to_string(&config_path)?)?;

    let ctx = create_context(&config).await?;
    let app = web::create_server(ctx);

    warp::serve(app).run(config.listen).await;
    Ok(())
}

async fn create_context(conf: &Config) -> anyhow::Result<CtxRef> {
    let site_config = util::load_yaml(conf.data_path.join("site.yml"))?;
    let main_page = util::load_markdown(conf.data_path.join("main_page.md"))?;
    let site_pages = SitePages { main: main_page };
    
    let main_db = conf.main_db.create_pool().await?;
    
    let mut campaigns = Vec::with_capacity(conf.campaigns.len());
    let mut campaigns_by_id = HashMap::with_capacity(conf.campaigns.len());

    for cconf in &conf.campaigns {
        info!("Loading campaign {:?}", &cconf.id);
        let campaign = framework::load_campaign(&conf.data_path, &cconf.id)?;
        let chars_db = cconf.chars_db.create_pool().await?;
        let ccontext = Arc::new(CampaignContext {
            data: campaign,
            chars_db
        });
        campaigns_by_id.insert(cconf.id.clone(), ccontext.clone());
        campaigns.push(ccontext);
    }

    Ok(Arc::new(Context {
        data_path: conf.data_path.clone(),
        site_config,
        site_pages,
        main_db,
        campaigns,
        campaigns_by_id
    }))
}
