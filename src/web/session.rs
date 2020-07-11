use time::Duration;
use cookie::{Cookie, SameSite};
use warp::{Filter, Rejection};
use warp::filters::BoxedFilter;
use crate::ctx;
use crate::error::AppError;
use crate::db;
use crate::db::account::Account;
use super::FilterResult;

pub const SESSION_COOKIE_NAME: &'static str = "session";

pub struct Session {
    pub key: String,
    pub account: Account,
}

impl Session {
    pub fn with<T>(self, reply: T) -> WithSession
    where
        T: warp::Reply + 'static
    {
        WithSession {
            session: Some(self),
            reply: Box::new(reply),
        }
    }
}

pub struct WithSession {
    pub session: Option<Session>,
    pub reply: Box<dyn warp::Reply>,
}

impl AsRef<Account> for Session {
    fn as_ref(&self) -> &Account {
        &self.account
    }
}

impl WithSession {
    pub fn none<T>(reply: T) -> Self
    where
        T: warp::Reply + 'static
    {
        Self {
            session: None,
            reply: Box::new(reply),
        }
    }

    pub fn reply(self) -> impl warp::Reply {
        let cookie = if let Some(session) = self.session {
            update_cookie(session.key)
        } else {
            remove_cookie()
        };
        warp::reply::with_header(self.reply, http::header::SET_COOKIE, cookie.to_string())
    }
}

pub fn fetch_session() -> BoxedFilter<(Option<Session>,)> {
    warp::cookie::optional(SESSION_COOKIE_NAME)
        .and_then(async move |maybe_key: Option<String>| -> FilterResult<Option<Session>> {
            if let Some(key) = maybe_key {
                db::session::touch(ctx().site_db.clone(), &key)
                    .await
                    .map(|opt| opt.map(|account| Session { key, account }))
                    .map_err(warp::Rejection::from)
            } else {
                Ok(None)
            }
        })
        .boxed()
}

pub fn fetch_session_required() -> BoxedFilter<(Session,)> {
    fetch_session()
        .and_then(async move |s: Option<Session>| -> FilterResult<Session> {
            s.ok_or(AppError::Unauthed.into())
        })
        .boxed()
}

pub fn unauthed_required() -> BoxedFilter<()> {
    fetch_session()
        .and_then(async move |s: Option<Session>| -> FilterResult<()> {
            if s.is_none() {
                Ok(())
            } else {
                Err(AppError::Forbidden.into())
            }
        })
        .untuple_one()
        .boxed()
}

fn update_cookie(session_key: String) -> Cookie<'static> {
    Cookie::build(SESSION_COOKIE_NAME, session_key)
        .path("/")
        .http_only(true)
        .secure(!cfg!(debug_assertions))
        .same_site(SameSite::Strict)
        .max_age(Duration::days(90))
        .finish()
}
    
pub fn remove_cookie() -> Cookie<'static> {
    Cookie::build(SESSION_COOKIE_NAME, "")
        .path("/")
        .http_only(true)
        .secure(!cfg!(debug_assertions))
        .same_site(SameSite::Strict)
        .max_age(Duration::zero())
        .finish()
}
