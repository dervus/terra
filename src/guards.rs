use rocket::Outcome;
use rocket::http::Status;
use rocket::request::{self, Request, FromRequest};
use anyhow::anyhow;
use crate::db::{AuthDB, AccountInfo, fetch_account_info};

const USER_ACCESS_LEVEL: u8 = 0;
const MODERATOR_ACCESS_LEVEL: u8 = 1;
const GAMEMASTER_ACCESS_LEVEL: u8 = 2;
const ADMIN_ACCESS_LEVEL: u8 = 3;

pub struct MaybeUser(pub Option<AccountInfo>);
pub struct User(pub AccountInfo);
pub struct Moderator(pub AccountInfo);
pub struct GameMaster(pub AccountInfo);
pub struct Admin(pub AccountInfo);

impl<'a, 'r> FromRequest<'a, 'r> for MaybeUser {
    type Error = anyhow::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let mut db = request.guard::<AuthDB>()
            .map_failure(|_| (Status::InternalServerError, anyhow!("unable to acquire auth db connection")))?;
        
        if let Some(account_id) = request.cookies().get_private("account_id").and_then(|c| c.value().parse().ok()) {
            match fetch_account_info(&mut *db, account_id) {
                Ok(info) => Outcome::Success(MaybeUser(info)),
                Err(e) => Outcome::Failure((Status::InternalServerError, e))
            }
        } else {
            Outcome::Success(MaybeUser(None))
        }
    }
}

fn check_user<'r>(request: &Request<'r>, min_access: u8) -> request::Outcome<AccountInfo, anyhow::Error> {
    let maybe_user = request.guard::<MaybeUser>()
        .map_failure(|_| (Status::InternalServerError, anyhow!("unable to acquire user info")))?;
    
    if let MaybeUser(Some(user)) = maybe_user {
        if user.access_level >= min_access {
            Outcome::Success(user)
        } else {
            Outcome::Failure((Status::Forbidden, anyhow!("forbidden")))
        }
    } else {
        Outcome::Failure((Status::Unauthorized, anyhow!("unauthorized")))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = anyhow::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        check_user(request, USER_ACCESS_LEVEL).map(|info| User(info))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Moderator {
    type Error = anyhow::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        check_user(request, MODERATOR_ACCESS_LEVEL).map(|info| Moderator(info))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for GameMaster {
    type Error = anyhow::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        check_user(request, GAMEMASTER_ACCESS_LEVEL).map(|info| GameMaster(info))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Admin {
    type Error = anyhow::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        check_user(request, ADMIN_ACCESS_LEVEL).map(|info| Admin(info))
    }
}
