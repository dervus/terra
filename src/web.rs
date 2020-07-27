use http::StatusCode;
use serde::Deserialize;
use serde_json::json;
use warp::{filters::BoxedFilter, reply::Json, Filter, Rejection, Reply};
use crate::{db, error, init::CtxRef};

type FilterResult<T> = Result<T, Rejection>;
type JsonResult = FilterResult<Json>;

fn with<T>(value: T) -> BoxedFilter<(T,)>
where
    T: 'static + Clone + Send + Sync,
{
    warp::any().map(move || value.clone()).boxed()
}

pub fn create_server(ctx: CtxRef) -> BoxedFilter<(impl Reply,)> {
    let campaign_read = warp::get()
        .and(warp::path!("campaign"))
        .and(with(ctx.clone()))
        .map(|ctx: CtxRef| {
            warp::reply::json(&json!({
                "blocks": &ctx.campaign.blocks,
                "role": &ctx.campaign.roles,
                "location": &ctx.campaign.system_view.location,
                "race": &ctx.campaign.system_view.race,
                "class": &ctx.campaign.system_view.class,
                "armor": &ctx.campaign.system_view.armor,
                "weapon": &ctx.campaign.system_view.weapon,
                "trait": &ctx.campaign.system_view.traits,
            }))
        });

    let account_read = warp::get()
        .and(warp::path!("accounts" / u32))
        .and(with(ctx.clone()))
        .and_then(account_read_handler);

    let account_create = warp::post()
        .and(warp::path!("accounts"))
        .and(warp::body::json())
        .and(with(ctx.clone()))
        .and_then(account_create_handler);

    let account_replace = warp::put()
        .and(warp::path!("accounts" / u32))
        .and(warp::body::json())
        .and(with(ctx.clone()))
        .and_then(account_replace_handler);

    let account_update = warp::patch()
        .and(warp::path!("accounts" / u32))
        .and(warp::body::json())
        .and(with(ctx.clone()))
        .and_then(account_update_handler);

    let character_create = warp::post()
        .and(warp::path!("characters"))
        .and(warp::body::json())
        .and(with(ctx.clone()))
        .and_then(character_create_handler);

    let character_list_mine = warp::get()
        .and(warp::path!("characters" / "mine" / u32))
        .and(with(ctx.clone()))
        .and_then(character_list_mine_handler);

    let character_list_other = warp::get()
        .and(warp::path!("characters" / "other" / u32))
        .and(with(ctx.clone()))
        .and_then(character_list_other_handler);

    let character_read = warp::get()
        .and(warp::path!("characters" / "guid" / u32))
        .and(with(ctx.clone()))
        .and_then(character_read_handler);

    let character_check_name = warp::post()
        .and(warp::path!("characters" / "check-name"))
        .and(warp::body::json())
        .and(with(ctx.clone()))
        .and_then(character_check_name_handler);

    campaign_read
        .or(account_read)
        .or(account_create)
        .or(account_replace)
        .or(account_update)
        .or(character_create)
        .or(character_list_mine)
        .or(character_list_other)
        .or(character_read)
        .or(character_check_name)
        .recover(error::handle_rejection)
        .with(warp::log::log("terra"))
        .boxed()
}

async fn account_read_handler(account: u32, ctx: CtxRef) -> JsonResult {
    let data = db::account::read(ctx.auth_db.clone(), account).await?;
    Ok(warp::reply::json(&data))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountCreate {
    username: String,
    password: String,
}

async fn account_create_handler(input: AccountCreate, ctx: CtxRef) -> JsonResult {
    let id = db::account::create(ctx.auth_db.clone(), &input.username, &input.password).await?;
    Ok(warp::reply::json(&json!({ "id": id })))
}

async fn account_replace_handler(
    account: u32,
    input: AccountCreate,
    ctx: CtxRef,
) -> FilterResult<impl Reply> {
    db::account::replace(
        ctx.auth_db.clone(),
        account,
        &input.username,
        &input.password,
    )
    .await?;
    Ok(warp::reply::with_status("", StatusCode::NO_CONTENT))
}

async fn account_update_handler(
    account: u32,
    input: AccountCreate,
    ctx: CtxRef,
) -> FilterResult<impl Reply> {
    db::account::update(
        ctx.auth_db.clone(),
        account,
        &input.username,
        &input.password,
    )
    .await?;
    Ok(warp::reply::with_status("", StatusCode::NO_CONTENT))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CharacterCreate {
    account: u32,
    #[serde(flatten)]
    form: db::character::Form,
}

async fn character_create_handler(input: CharacterCreate, ctx: CtxRef) -> JsonResult {
    let cdata = input.form.into_cdata(&ctx.campaign)?;
    let guid = db::character::create(ctx.chars_db.clone(), input.account, cdata).await?;
    Ok(warp::reply::json(&json!({ "guid": guid })))
}

async fn character_list_mine_handler(account: u32, ctx: CtxRef) -> JsonResult {
    let data = db::character::list_mine(ctx.chars_db.clone(), account).await?;
    Ok(warp::reply::json(&data))
}

async fn character_list_other_handler(account: u32, ctx: CtxRef) -> JsonResult {
    let data = db::character::list_other(ctx.chars_db.clone(), account).await?;
    Ok(warp::reply::json(&data))
}

async fn character_read_handler(guid: u32, ctx: CtxRef) -> JsonResult {
    let data = db::character::read(ctx.chars_db.clone(), guid).await?;
    Ok(warp::reply::json(&data))
}

#[derive(Deserialize)]
struct CheckName {
    name: String,
}

async fn character_check_name_handler(input: CheckName, ctx: CtxRef) -> JsonResult {
    let data = db::character::check_name(ctx.chars_db.clone(), &input.name).await?;
    Ok(warp::reply::json(&json!({ "result": data })))
}
