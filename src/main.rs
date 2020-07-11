#![feature(proc_macro_hygiene, async_closure)]

mod error;
mod init;
mod util;
mod db;
mod framework;
mod view;
mod web;

use std::sync::{Arc, Weak};
use log::info;
use async_ctrlc::CtrlC;
use crate::init::{AppContext, CtxRef, CCtxRef};
use crate::error::{AppError, AppResult};

static mut CONTEXT: Option<Weak<AppContext>> = None;

fn ctx() -> CtxRef {
    unsafe {
        CONTEXT.clone().unwrap().upgrade().unwrap()
    }
}

fn site_db() -> sqlx::postgres::PgPool {
    ctx().site_db.clone()
}

fn campaign<T>(id: T) -> AppResult<CCtxRef> where T: AsRef<str> {
    ctx().campaigns_by_id.get(id.as_ref()).cloned().ok_or(AppError::NotFound)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let ctrlc = CtrlC::new()?;

    let mut args = std::env::args();
    args.next();
    let config_path = args.next().unwrap_or("config.yml".to_owned());

    info!("Loading config file {}", &config_path);
    let config: init::AppConfig = serde_yaml::from_str(&std::fs::read_to_string(&config_path)?)?;

    let listen = config.listen.clone();
    let ctx = init::create_context(config).await?;

    unsafe {
        CONTEXT = Some(Arc::downgrade(&ctx));
    }

    let app = web::create_server();
    warp::serve(app).bind_with_graceful_shutdown(listen, ctrlc).1.await;

    info!("Exiting...");
    Ok(())
}
