use thiserror::Error;
use crate::db;

#[derive(Debug, Error)]
pub enum TerraError {
    #[error("")]
    InvalidInput,
    #[error("")]
    InvalidSession,
    #[error("")]
    LowAccessLevel,
    #[error("")]
    RedisPoolFailure(#[from] mobc::Error<db::RedisError>),
    #[error("")]
    RedisFailure(#[from] db::RedisError),
    #[error("")]
    MysqlFailure(#[from] db::MysqlError),
    #[error("")]
    Other(#[from] anyhow::Error),
}

impl From<TerraError> for warp::Rejection {
    fn from(error: TerraError) -> Self {
        warp::reject::custom(error)
    }
}

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type TerraResult<T> = StdResult<T, TerraError>;

impl warp::reject::Reject for TerraError {}

pub fn invalid_input() -> warp::Rejection {
    warp::reject::custom(TerraError::InvalidInput)
}

pub fn invalid_session() -> warp::Rejection {
    warp::reject::custom(TerraError::InvalidSession)
}

pub fn low_access_level() -> warp::Rejection {
    warp::reject::custom(TerraError::LowAccessLevel)
}

pub fn other(error: anyhow::Error) -> warp::Rejection {
    warp::reject::custom(TerraError::Other(error))
}
