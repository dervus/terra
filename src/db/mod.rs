use std::time::Duration;
use serde::Deserialize;

fn default_min_connections() -> u32 { 1 }
fn default_max_connections() -> u32 { 10 }
fn default_connect_timeout() -> Duration { Duration::from_secs(60) }
fn default_idle_timeout() -> Option<Duration> { None }
fn default_max_lifetime() -> Option<Duration> { Some(Duration::from_secs(1800)) }

#[derive(Deserialize)]
pub struct DBConfig {
    pub connect: String,
    #[serde(default = "default_min_connections")] pub min_connections: u32,
    #[serde(default = "default_max_connections")] pub max_connections: u32,
    #[serde(default = "default_connect_timeout")] pub connect_timeout: Duration,
    #[serde(default = "default_idle_timeout")] pub idle_timeout: Option<Duration>,
    #[serde(default = "default_max_lifetime")] pub max_lifetime: Option<Duration>,
}

impl DBConfig {
    pub async fn create_pool<DB: sqlx::Database>(&self) -> anyhow::Result<sqlx::Pool<DB>> {
        sqlx::pool::PoolOptions::new()
            .min_connections(self.min_connections)
            .max_connections(self.max_connections)
            .connect_timeout(self.connect_timeout)
            .max_lifetime(self.max_lifetime)
            .idle_timeout(self.idle_timeout)
            .connect(&self.connect)
            .await
            .map_err(From::from)
    }
}

pub mod account;
pub mod character;
