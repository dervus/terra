use thiserror::Error;

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
