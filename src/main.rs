#![feature(async_closure)]

mod db;
mod error;
mod framework;
mod init;
mod util;
mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let ctrlc = async_ctrlc::CtrlC::new()?;

    let mut args = std::env::args();
    args.next();
    let config_path = args.next().unwrap_or("config.yml".to_owned());
    let config: init::AppConfig =
        tokio::task::spawn_blocking(move || util::load_yaml(config_path)).await??;

    let listen = config.listen.clone();
    let ctx = tokio::task::spawn_blocking(move || init::create_context(config)).await??;
    let app = web::create_server(ctx);

    warp::serve(app)
        .bind_with_graceful_shutdown(listen, ctrlc)
        .1
        .await;

    log::info!("Cleaning up before exit...");
    Ok(())
}
