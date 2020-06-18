use thiserror::Error;
use crate::db;

#[derive(Debug, Error)]
pub enum WebError {
    #[error("")]
    InvalidInput,
    #[error("")]
    InvalidSession,
    #[error("")]
    LowAccessLevel,
    #[error("")]
    DBError(#[from] db::DBError),
    #[error("")]
    Other(#[from] anyhow::Error),
}

impl From<WebError> for warp::Rejection {
    fn from(error: WebError) -> Self {
        warp::reject::custom(error)
    }
}

impl From<db::DBError> for warp::Rejection {
    fn from(error: db::DBError) -> Self {
        warp::reject::custom(WebError::from(error))
    }
}

pub type WebResult<T> = Result<T, WebError>;

impl warp::reject::Reject for WebError {}

pub fn invalid_input() -> warp::Rejection {
    warp::reject::custom(WebError::InvalidInput)
}

pub fn invalid_session() -> warp::Rejection {
    warp::reject::custom(WebError::InvalidSession)
}

pub fn low_access_level() -> warp::Rejection {
    warp::reject::custom(WebError::LowAccessLevel)
}

pub fn other(error: anyhow::Error) -> warp::Rejection {
    warp::reject::custom(WebError::Other(error))
}
