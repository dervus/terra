pub mod session;
mod auth;
mod pc;

use std::net::{SocketAddr};
use http::StatusCode;
use warp::{Filter, Reply, Rejection};
use warp::filters::BoxedFilter;
use crate::{ctx, campaign};
use crate::view;
use self::session::Session;
use crate::view::Page;
use crate::init::{CtxRef, CCtxRef};

pub type FilterResult<T> = Result<T, Rejection>;

fn with<T>(value: T) -> BoxedFilter<(T,)>
where
    T: 'static + Clone + Send + Sync,
{
    warp::any().map(move || value.clone()).boxed()
}

fn campaign_param() -> BoxedFilter<(CCtxRef,)> {
    warp::path::param()
        .and_then(async move |id: String| campaign(id).map_err(Rejection::from))
        .boxed()
}

pub fn create_server() -> BoxedFilter<(impl Reply,)> {
    let assets = warp::get()
        .and(warp::path("assets"))
        .and(warp::fs::dir(ctx().assets_path.clone()));

    let campaign_front_page = warp::get()
        .and(campaign_param())
        .and(warp::path::end())
        .and(session::fetch_session())
        .map(|cgn: CCtxRef, session: Option<Session>| view::index::campaign(&cgn.data).render(session.as_ref()));

    let root = warp::get()
        .and(warp::path::end())
        .and(session::fetch_session())
        .map(|session: Option<Session>| Page::untitled().content(maud::PreEscaped(ctx().site_pages.front.clone())).render(session.as_ref()));

    let login_page = warp::get()
        .and(warp::path!("login"))
        .and(session::unauthed_required())
        .and(warp::cookie::optional("captcha").map(|c: Option<String>| c.map(|s| s == "true").unwrap_or(false)))
        .map(auth::login_page);

    let login_action = warp::post()
        .and(warp::path!("login"))
        .and(session::unauthed_required())
        .and(warp::body::form())
        .and(warp::addr::remote().map(|a: Option<SocketAddr>| a.map(|a| a.ip())))
        .and(warp::cookie::optional("redirect"))
        .and_then(auth::login_action);

    let logout_action = warp::post()
        .and(warp::path!("logout"))
        .and(session::fetch_session_required())
        .and_then(auth::logout_action);

    let register_page = warp::get()
        .and(warp::path!("register"))
        .and(session::unauthed_required())
        .map(auth::register_page);

    let register_action = warp::post()
        .and(warp::path!("register"))
        .and(session::unauthed_required())
        .and(warp::body::form())
        .and(warp::addr::remote().map(|a: Option<SocketAddr>| a.map(|a| a.ip())))
        .and(warp::cookie::optional("redirect"))
        .and_then(auth::register_action);

    let pc_new_page = warp::get()
        .and(warp::path("characters"))
        .and(warp::path("new"))
        .and(campaign_param())
        .and(warp::path::end())
        .and(session::fetch_session_required())
        .map(pc::new_page);

    assets
        .or(campaign_front_page)
        .or(root)
        .or(login_page)
        .or(login_action)
        .or(logout_action)
        .or(register_page)
        .or(register_action)
        .or(pc_new_page)
        .recover(crate::error::handle_rejection)
        .boxed()
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

    // let login_action = session_update_reply(
    //     warp::post()
    //         .and(warp::path!("login"))
    //         .and(warp::body::form())
    //         .and(with_context())
    //         .and_then(post_login)
    //         .boxed());

    // login_action

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
