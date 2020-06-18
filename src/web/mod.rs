mod error;
mod session;
mod page;

use std::path::PathBuf;
use std::sync::Arc;
use http::{Response, StatusCode};
use warp::{Filter, Reply, Rejection};
use warp::filters::BoxedFilter;
use serde::Deserialize;
use crate::view;
use crate::db;
use crate::{CtxRef, CCtxRef};
use self::session::{Session, WithSession};
use self::page::Page;

type FilterResult<T> = Result<T, Rejection>;

struct Handler {
    ctx: CtxRef,
    session: Option<Session>,
    session_update: bool,
    reply: HandlerReply,
}

enum HandlerReply {
    Page(Page),
    Other(Box<dyn Reply>),
}

impl From<Page> for HandlerReply {
    fn from(page: Page) -> Self { Self::Page(page) }
}

impl<T: Reply + 'static> From<T> for HandlerReply {
    fn from(reply: T) -> Self { Self::Other(Box::new(reply)) }
}

impl Reply for Handler {
    fn into_response(self) -> warp::reply::Response {
        let reply: Box<dyn Reply> = match self.reply {
            HandlerReply::Page(page) => {
                page.site_title = self.ctx.site_config.title.clone();
                Box::new(warp::reply::html(page.into_markup().into_string()))
            },
            HandlerReply::Other(other) => other,
        };
        if let Some(session) = self.session {
            warp::reply::with_header(reply, http::header::SET_COOKIE, "asd").into_response()
        } else {
            reply.into_response()
        }
    }
}

fn with_context(ctx: CtxRef) -> BoxedFilter<(CtxRef,)> {
    warp::any().map(move || ctx.clone()).boxed()
}

fn with_session(ctx: CtxRef) -> BoxedFilter<(WithSession,)> {
    with_context(ctx)
        .and(warp::cookie::optional(session::COOKIE_NAME))
        .and_then(async move |ctx: CtxRef, encoded_key: Option<String>| -> FilterResult<WithSession> {
            let decoded_key = encoded_key.and_then(|k| db::session::decode_key(&k).map(Some).unwrap_or(None));
            if let Some(key) = decoded_key {
                let account_id = db::session::touch(ctx.main_db.clone(), &key).await.map_err(Rejection::from)?;
                let account = db::account::read(ctx.main_db.clone(), account_id).await.map_err(Rejection::from)?;
                let session = Session { key, account };
                Ok(WithSession::new().init(Some(session)))
            } else {
                Ok(WithSession::new())
            }
        })
        .boxed()
}

fn post_login(ctx: CtxRef) -> BoxedFilter<(impl Reply,)> {
    #[derive(Deserialize)]
    struct Form { name: String, password: String }
    with_context(ctx)
        .and(warp::post())
        .and(warp::path!("login"))
        .and(warp::body::form())
        .and_then(async move |ctx: CtxRef, form: Form| -> FilterResult<WithSession> {
            let maybe_account = db::account::login(ctx.main_db.clone(), &form.name, &form.password).await.map_err(Rejection::from)?;
            if let Some(account) = maybe_account {
                let key = db::session::create(ctx.main_db.clone(), account.account_id).await.map_err(Rejection::from)?;
                Ok(WithSession::new().update(Some(Session { key, account })))
            } else {
                Ok(WithSession::new())
            }
        })
        .map(|ws: WithSession| if let Some(session) = &ws.session {
            ws.with(Page::new()
                    .title("Вход")
                    .account(Some(&session.account))
                    .redirect(5, "/"))
        } else {
            ws.with(Page::new()
                    .title("Вход")
                    .redirect(5, "/fuck/you"))
        })
        .boxed()
}

pub fn create_server(ctx: CtxRef) -> BoxedFilter<(impl Reply,)> {
    warp::any().map(|| "Hello World!").boxed()
    // let with_ctx = warp::any().map(move || ctx.clone());

    // // utility combinators
    // let with_page = warp::cookie::cookie("session")
    //     .and(with_redis_conn.clone())
    //     .and_then(async move |session_key: String, mut conn: RedisConn| -> FilterResult<(String, SessionData)> {
    //         let maybe_session_data = db::fetch_session_data(&mut conn, &session_key).await?;
    //         let session_data = maybe_session_data.ok_or(TerraError::InvalidSession)?;
    //         Ok((session_key, session_data))
    //     })
    //     .and(with_auth_conn.clone())
    //     .and_then(async move |input: (String, SessionData), conn: MysqlConn| -> FilterResult<(String, AccountInfo)> {
    //         let (session_key, session_data) = input;
    //         let (_, maybe_account) = db::fetch_account_info(conn, session_data.account_id).await?;
    //         let account = maybe_account.ok_or(TerraError::InvalidSession)?;
    //         Ok((session_key, account))
    //     })
    //     .map(|s| Some(s))
    //     .or(warp::any().map(|| None))
    //     .unify()
    //     .map(|input: Option<(String, AccountInfo)>| if let Some((key, account)) = input {
    //         Page::new().session(Session::LoggedIn(key, account))
    //     } else {
    //         Page::new()
    //     });

    // let with_authed_page = with_page.clone()
    //     .and_then(async move |page: Page| -> FilterResult<(Page, AccountInfo)> {
    //         if let Session::LoggedIn(_, ref account) = page.session {
    //             let cloned_account = account.clone();
    //             Ok((page, cloned_account))
    //         } else {
    //             Err(low_access_level()) // FIXME
    //         }
    //     })
    //     .untuple_one();
    
    // // handlers
    // let static_files = warp::get()
    //     .and(warp::path("static"))
    //     .and(warp::fs::dir(config.files_path.clone()));
    
    // let asset_files = warp::get()
    //     .and(warp::path("assets"))
    //     .and(warp::fs::dir(config.data_path.join("assets")));

    // let signup_page = warp::get()
    //     .and(warp::path!("signup"))
    //     .and(with_page.clone())
    //     .map(|page: Page| page.title("Регистрация").content(views::signup()));

    // let login_page = warp::get()
    //     .and(warp::path!("login"))
    //     .and(with_page.clone())
    //     .map(|page: Page| page.title("Вход").content(views::login()));

    // #[derive(Deserialize)]
    // struct LoginForm {
    //     name: String,
    //     password: String,
    // }
    // let login_action = warp::post()
    //     .and(warp::path!("login"))
    //     .and(warp::body::form())
    //     .and(with_page.clone())
    //     .and(with_auth_conn.clone())
    //     .and(with_redis_conn.clone())
    //     .and_then(async move |form: LoginForm, page: Page, mysql: MysqlConn, mut redis: RedisConn| -> FilterResult<Page> {
    //         if let (_, Some(account)) = db::login_query(mysql, &form.name, &form.password).await.map_err(other)? {
    //             let key = db::create_session_data(&mut redis, &SessionData { account_id: account.id }).await.map_err(other)?;
    //             Ok(page
    //                .status(StatusCode::CREATED)
    //                .session(Session::LoggedIn(key, account))
    //                .redirect(0, "/")
    //                .content(views::redirect_page("/")))
    //         } else {
    //             Ok(page
    //                .status(StatusCode::FORBIDDEN)
    //                .content(views::login()))
    //         }
    //     });

    // let logout_action = warp::post()
    //     .and(warp::path!("logout"))
    //     .and(with_page.clone())
    //     .and(with_redis_conn.clone())
    //     .and_then(async move |page: Page, mut conn: RedisConn| {
    //         match &page.session {
    //             Session::LoggedIn(key, _) => {
    //                 db::delete_session_data(&mut conn, key).await.map_err(other)?;
    //                 Ok(page
    //                    .session(Session::JustLoggedOut)
    //                    .redirect(0, "/")
    //                    .content(views::redirect_page("/")))
    //             }
    //             _ => Err(invalid_session())
    //         }
    //     });

    // let forum_page = warp::get()
    //     .and(warp::path!("forum"))
    //     .and(with_page.clone())
    //     .map(|page: Page| page.title("Форум").content(views::forum_page()));

    // let campaign_page = warp::get()
    //     .and(warp::path::end())
    //     .and(with_page.clone())
    //     .and(with_campaign.clone())
    //     .map(|page: Page, campaign: Arc<Campaign>| page.content(views::campaign_page(&campaign)));

    // let roles_page = warp::get()
    //     .and(warp::path!("roles"))
    //     .and(with_page.clone())
    //     .and(with_campaign.clone())
    //     .map(|page: Page, campaign: Arc<Campaign>| page.title("Роли").content(views::roles_page(&campaign)));

    // let character_index = warp::get()
    //     .and(warp::path!("characters"))
    //     .and(with_page.clone())
    //     .and(with_campaign.clone())
    //     .and(with_chars_conn.clone())
    //     .and_then(async move |page: Page, campaign: Arc<Campaign>, conn: MysqlConn| -> FilterResult<Page> {
    //         let (_, list) = db::list_characters(conn, campaign.as_ref()).await.map_err(other)?;
    //         Ok(page.title("Персонажи").content(views::character_index(&campaign, &list)))
    //     });

    // let character_form = warp::get()
    //     .and(warp::path!("characters" / "new"))
    //     .and(with_page.clone())
    //     .and(with_campaign.clone())
    //     .map(|page: Page, campaign: Arc<Campaign>| {
    //         page.title("Новый персонаж")
    //             .stylesheet("/static/css/character_form.css")
    //             .script("/static/js/character_form.js")
    //             .content(views::character_form(&campaign, None))
    //     });

    // let character_form_with_role = warp::get()
    //     .and(warp::path!("characters" / "new" / String))
    //     .and(with_page.clone())
    //     .and(with_campaign.clone())
    //     .map(|role: String, page: Page, campaign: Arc<Campaign>| {
    //         page.title("Новый персонаж")
    //             .stylesheet("/static/css/character_form.css")
    //             .script("/static/js/character_form.js")
    //             .content(views::character_form(&campaign, Some(&role)))
    //     });
    
    // // let insert_character = warp::post()
    // //     .and(warp::path!("characters" / "new"))
    // //     .and(warp::body::form())
    // //     .and(require_account.clone())
    // //     .and(with_campaign.clone())
    // //     .and(with_chars_db.clone())
    // //     .and_then(async move |form: Vec<(String, String)>, account: db::AccountInfo, campaign: Arc<Campaign>, conn: MysqlConn| -> Result<warp::reply::Response, warp::Rejection> {
    // //         let _guid = handlers::insert_character(conn, &account, campaign.as_ref(), None, &form).await.map_err(|e| warp::reject::custom(GenericFailure(e)))?;
    // //         Ok(warp::reply::with_status(warp::reply::html(render::redirect_page("/characters").into_string()), http::StatusCode::BAD_REQUEST).into_response())
    // //     });

    // // let insert_character_with_role = warp::post()
    // //     .and(warp::path!("characters" / "new" / String))
    // //     .and(warp::body::form())
    // //     .and(require_account.clone())
    // //     .and(with_campaign.clone())
    // //     .and(with_chars_db.clone())
    // //     .and_then(async move |role: String, form: Vec<(String, String)>, account: db::AccountInfo, campaign: Arc<Campaign>, conn: MysqlConn| -> Result<warp::reply::Response, warp::Rejection> {
    // //         let _guid = handlers::insert_character(conn, &account, campaign.as_ref(), Some(role), &form).await.map_err(|e| warp::reject::custom(GenericFailure(e)))?;
    // //         Ok(warp::reply::with_status(warp::reply::html(render::redirect_page("/characters").into_string()), http::StatusCode::BAD_REQUEST).into_response())
    // //     });
    
    // // bundling everything together
    // static_files
    //     .or(asset_files)
    //     .or(signup_page)
    //     .or(login_page)
    //     .or(login_action)
    //     .or(logout_action)
    //     .or(forum_page)
    //     .or(campaign_page)
    //     .or(roles_page)
    //     .or(character_index)
    //     .or(character_form)
    //     .or(character_form_with_role)
    //     //.or(insert_character)
    //     //.or(insert_character_with_role)
    //     .with(warp::log::log("terra"))
    //     .boxed()
}
