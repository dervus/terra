use std::convert::Infallible;
use http::StatusCode;
use serde_json::json;
use thiserror::Error;
use warp::{Rejection, Reply, reject::Reject};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound,
    #[error("resource already exists")]
    Conflict,
    #[error("invalid request input: {0}")]
    InvalidInput(&'static str),
    #[error("database adapter error")]
    DatabaseError(sqlx::Error),
    #[error("unknown error")]
    Other(#[from] anyhow::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let Some(db_err) = err.as_database_error() {
            if let Some(code) = db_err.code() {
                if code == "23000" {
                    return AppError::Conflict;
                }
            }
        }
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound,
            _ => AppError::DatabaseError(err),
        }
    }
}

impl From<AppError> for warp::Rejection {
    fn from(error: AppError) -> Self {
        warp::reject::custom(error)
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl Reject for AppError {}

pub async fn handle_rejection(rej: Rejection) -> Result<impl Reply, Infallible> {
    if rej.is_not_found() {
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({ "error": "not_found" })),
            StatusCode::NOT_FOUND,
        ))
    } else if let Some(err) = rej.find::<AppError>() {
        match err {
            AppError::NotFound => Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "not_found",
                })),
                StatusCode::NOT_FOUND,
            )),
            AppError::Conflict => Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "conflict",
                })),
                StatusCode::CONFLICT,
            )),
            AppError::InvalidInput(reason) => Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "bad_request",
                    "cause": reason,
                })),
                StatusCode::BAD_REQUEST,
            )),
            AppError::DatabaseError(inner) => Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "internal_server_error",
                    "cause": format!("{:?}", inner),
                })),
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
            AppError::Other(inner) => Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "error": "internal_server_error",
                    "cause": format!("{:?}", inner),
                })),
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    } else if let Some(err) = rej.find::<warp::reject::InvalidHeader>() {
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "error": "bad_request",
                "cause": err.name(),
            })),
            StatusCode::BAD_REQUEST,
        ))
    } else if let Some(err) = rej.find::<warp::reject::MissingCookie>() {
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "error": "bad_request",
                "cause": err.name(),
            })),
            StatusCode::BAD_REQUEST,
        ))
    } else if let Some(err) = rej.find::<warp::reject::MissingHeader>() {
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "error": "bad_request",
                "cause": err.name(),
            })),
            StatusCode::BAD_REQUEST,
        ))
    } else if let Some(err) = rej.find::<warp::filters::body::BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "error": "bad_request",
                "cause": err.to_string(),
            })),
            StatusCode::BAD_REQUEST,
        ))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "error": "internal_server_error",
                "cause": format!("{:?}", rej),
            })),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
