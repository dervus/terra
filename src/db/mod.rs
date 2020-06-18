use std::time::Duration;
use serde::Deserialize;
use thiserror::Error;

fn default_min_size() -> u32 { 0 }
fn default_max_size() -> u32 { 10 }
fn default_connect_timeout() -> Duration { Duration::from_secs(60) }
fn default_idle_timeout() -> Option<Duration> { None }
fn default_max_lifetime() -> Option<Duration> { Some(Duration::from_secs(1800)) }

#[derive(Deserialize)]
pub struct DBConfig {
    pub url: String,
    #[serde(default = "default_min_size")] pub min_size: u32,
    #[serde(default = "default_max_size")] pub max_size: u32,
    #[serde(default = "default_connect_timeout")] pub connect_timeout: Duration,
    #[serde(default = "default_idle_timeout")] pub idle_timeout: Option<Duration>,
    #[serde(default = "default_max_lifetime")] pub max_lifetime: Option<Duration>,
}

impl DBConfig {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            url: url.into(),
            min_size: default_min_size(),
            max_size: default_max_size(),
            connect_timeout: default_connect_timeout(),
            idle_timeout: default_idle_timeout(),
            max_lifetime: default_max_lifetime(),
        }
    }

    pub async fn create_pool<C: sqlx::Connect>(&self) -> anyhow::Result<sqlx::Pool<C>> {
        sqlx::Pool::builder()
            .min_size(self.min_size)
            .max_size(self.max_size)
            .connect_timeout(self.connect_timeout)
            .max_lifetime(self.max_lifetime)
            .idle_timeout(self.idle_timeout)
            .build(&self.url)
            .await
            .map_err(From::from)
    }
}

#[derive(Debug, Error)]
pub enum DBError {
    #[error("")]
    PasswordError(#[from] argon2::Error),
    #[error("")]
    AdapterError(#[from] sqlx::Error),
    #[error("")]
    Other(#[from] anyhow::Error),
}

pub type DBResult<T> = Result<T, DBError>;

pub mod account;
pub mod session;
// pub mod character;
