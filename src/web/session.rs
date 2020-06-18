use time::Duration;
use cookie::{Cookie, SameSite};
use crate::db;
use crate::db::account::Account;

pub const COOKIE_NAME: &'static str = "session";

pub struct Session {
    pub key: Vec<u8>,
    pub account: Account,
}

impl AsRef<Account> for Session {
    fn as_ref(&self) -> &Account {
        &self.account
    }
}

pub struct WithSession {
    pub update: bool,
    pub session: Option<Session>,
    pub reply: Box<dyn warp::Reply>,
}

impl WithSession {
    pub fn new() -> Self {
        Self {
            update: false,
            session: None,
            reply: Box::new(warp::reply::with_status("", http::StatusCode::NO_CONTENT)),
        }
    }

    pub fn init(self, session: Option<Session>) -> Self {
        self.session = session;
        self
    }

    pub fn touch(self) -> Self {
        self.update = true;
        self
    }

    pub fn update(self, session: Option<Session>) -> Self {
        self.session = session;
        self.touch()
    }

    pub fn with<T, I>(self, reply: I) -> Self
    where
        T: warp::Reply + 'static,
        I: Into<Box<T>>,
    {
        self.reply = reply.into();
        self
    }
}

impl warp::Reply for WithSession {
    fn into_response(self) -> warp::reply::Response {
        if self.update {
            let cookie = if let Some(session) = self.session {
                update_session_cookie(&session.key)
            } else {
                remove_session_cookie()
            };
            warp::reply::with_header(self.reply, http::header::SET_COOKIE, cookie.to_string()).into_response()
        } else {
            self.reply.into_response()
        }
    }
}

fn update_session_cookie(session_key: &[u8]) -> Cookie<'static> {
    Cookie::build(COOKIE_NAME, db::session::encode_key(session_key))
        .path("/")
        .http_only(true)
        .secure(!cfg!(debug_assertions))
        .same_site(SameSite::Strict)
        .max_age(Duration::days(90))
        .finish()
}
    
fn remove_session_cookie() -> Cookie<'static> {
    Cookie::build(COOKIE_NAME, "")
        .path("/")
        .http_only(true)
        .secure(!cfg!(debug_assertions))
        .same_site(SameSite::Strict)
        .max_age(Duration::zero())
        .finish()
}
