use std::convert::Infallible;
use thiserror::Error;
use warp::{Reply, Rejection};
use warp::reject::Reject;
use http::StatusCode;
use crate::view::SpecialPage;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound,
    #[error("invalid request inputs")]
    BadRequest,
    #[error("must be logged in")]
    Unauthed,
    #[error("lacking access")]
    Forbidden,
    #[error("database adapter error")]
    DatabaseError(sqlx::Error),
    #[error("unknown error")]
    Other(#[from] anyhow::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => AppError::NotFound,
            _ => AppError::DatabaseError(error),
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
        Ok(render(&AppError::NotFound))
    } else if let Some(err) = rej.find::<AppError>() {
        Ok(render(err))
    } else {
        Ok(render(&AppError::Other(anyhow::anyhow!("{:#?}", rej))))
    }
}

fn render(error: &AppError) -> impl Reply {
    let status = match error {
        AppError::NotFound => StatusCode::NOT_FOUND,
        AppError::BadRequest => StatusCode::BAD_REQUEST,
        AppError::Unauthed => StatusCode::UNAUTHORIZED,
        AppError::Forbidden => StatusCode::FORBIDDEN,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let message = match error {
        AppError::NotFound => "Здесь такого нет!",
        AppError::BadRequest => "Некорректный запрос",
        AppError::Unauthed => "Требуется вход в систему",
        AppError::Forbidden => "Недостаточный уровень доступа",
        _ => "Ошибка сервера",
    };
    let message = maud::html! { p { (message) } };
    Ok(SpecialPage::new(status).message(message).error(format!("{:#?}", error)).render())
}
