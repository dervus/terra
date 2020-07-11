use std::net::IpAddr;
use serde::{Serialize, Deserialize};
use warp::{Reply, Rejection};
use http::StatusCode;
use log::debug;
use crate::{ctx, site_db};
use crate::error::{AppError, AppResult};
use crate::init::CtxRef;
use crate::view::{self, Page, SpecialPage};
use crate::db;
use crate::db::account::LoginOutcome;
use super::FilterResult;
use super::session::{Session, WithSession};

#[derive(Deserialize)]
pub struct LoginForm {
    nick_or_email: String,
    password: String,
    #[serde(default, rename = "g-recaptcha-response")] captcha_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    nick: String,
    email: String,
    password: String,
    password_confirm: String,
    #[serde(rename = "g-recaptcha-response")] captcha_token: String,
}

pub fn login_page(show_captcha: bool) -> impl Reply {
    view::auth::login(show_captcha).render(None)
}

pub async fn login_action(form: LoginForm, user_addr: Option<IpAddr>, redirect: Option<String>) -> FilterResult<impl Reply> {
    let captcha_validated = if let Some(token) = form.captcha_token {
        verify_captcha(&token, user_addr).await?
    } else {
        false
    };

    let login_res = db::account::login(
        site_db(),
        &form.nick_or_email,
        &form.password,
        captcha_validated)
        .await?;

    debug!("login result: {:?}", &login_res);
    match login_res {
        LoginOutcome::NotFound | LoginOutcome::WrongPassword =>
            Err(AppError::BadRequest.into()),

        LoginOutcome::CaptchaRequired =>
            Ok(Box::new(warp::reply::with_header(view::auth::login(true).render(None), http::header::SET_COOKIE, cookie::Cookie::build("captcha", "true").finish().to_string())) as Box<dyn Reply>),

        LoginOutcome::Success(account) => {
            let key = db::session::create(site_db(), account.account_id).await?;
            let session = Session { key, account };
            let reply = view::index::redirect(StatusCode::CREATED, redirect.as_deref().unwrap_or("/"));
            Ok(Box::new(session.with(reply).reply()) as Box<dyn Reply>)
        }
    }
}

pub async fn logout_action(session: Session) -> FilterResult<impl Reply> {
    db::session::delete(site_db(), &session.key).await?;
    Ok(WithSession::none(view::index::redirect(StatusCode::OK, "/")).reply())
}

pub fn register_page() -> impl Reply {
    view::auth::register().render(None)
}

pub async fn register_action(form: RegisterForm, user_addr: Option<IpAddr>, redirect: Option<String>) -> FilterResult<impl Reply> {
    if form.password != form.password_confirm {
        return Err(AppError::BadRequest.into());
    }
    if !verify_captcha(&form.captcha_token, user_addr).await? {
        return Err(AppError::BadRequest.into());
    }
    let account = db::account::create(site_db(), &form.email, &form.nick, &form.password).await?;
    let key = db::session::create(site_db(), account.account_id).await?;
    let session = Session { key, account };
    let reply = view::index::redirect(StatusCode::CREATED, redirect.as_deref().unwrap_or("/"));
    Ok(session.with(reply).reply())
}

async fn verify_captcha(token: &str, user_addr: Option<IpAddr>) -> AppResult<bool> {
    #[derive(Serialize)]
    struct Form {
        secret: String,
        response: String,
        remoteip: Option<String>,
    }
    #[derive(Debug, Deserialize)]
    struct Response {
        success: bool,
        #[serde(default)] challenge_ts: Option<String>,
        #[serde(default)] hostname: Option<String>,
        #[serde(default, rename = "error-codes")] error_codes: Vec<String>,
    }
    let res: Response = ctx().http_client.post("https://www.google.com/recaptcha/api/siteverify")
        .form(&Form {
            secret: ctx().recaptcha.secret.clone(),
            response: token.to_owned(),
            remoteip: user_addr.map(|a| a.to_string()),
        })
        .send()
        .await
        .map_err(anyhow::Error::from)?
        .json()
        .await
        .map_err(anyhow::Error::from)?;
    debug!("recaptcha response: {:?}", &res);
    Ok(res.success)
}

#[derive(Serialize, Deserialize)]
struct SignedData {
    tag: Vec<u8>,
    data: Vec<u8>,
}

fn pack_signed_data<T: Serialize>(payload: &T, secret: &ring::hmac::Key) -> AppResult<String> {
    let data = bincode::serialize(payload).map_err(anyhow::Error::from)?;
    let tag = ring::hmac::sign(secret, &data).as_ref().to_owned();
    let bundle = bincode::serialize(&SignedData { tag, data }).map_err(anyhow::Error::from)?;
    Ok(base64::encode_config(&bundle, base64::URL_SAFE_NO_PAD))
}

fn unpack_signed_data<T: serde::de::DeserializeOwned>(payload: &str, secret: &ring::hmac::Key) -> AppResult<T> {
    let bundle_bytes = base64::decode_config(payload, base64::URL_SAFE_NO_PAD).map_err(anyhow::Error::from)?;
    let bundle: SignedData = bincode::deserialize(&bundle_bytes).map_err(anyhow::Error::from)?;
    ring::hmac::verify(secret, &bundle.data, &bundle.tag).map_err(|_| anyhow::anyhow!("wrong signature"))?;
    let data: T = bincode::deserialize(&bundle.data).map_err(anyhow::Error::from)?;
    Ok(data)
}
