use rocket::Route;
use rocket::request::Form;
use rocket::response::status;
use rocket::http::{Cookies, Cookie, Status, RawStr};
use ring::digest;
use maud::{Markup, html};
use anyhow::Result;
use crate::guards::MaybeUser;
use crate::render::Page;
use crate::views;
use crate::db;

#[get("/")]
pub fn index(user: MaybeUser) -> Markup {
    Page::new()
        .account(user.0.as_ref())
        .render(html! { "Hello world!" })
}

#[get("/signup")]
pub fn signup(user: MaybeUser) -> Markup {
    Page::new()
        .title("Регистрация")
        .account(user.0.as_ref())
        .render(views::signup())
}

#[get("/login")]
pub fn login_page() -> Markup {
    Page::new()
        .title("Вход")
        .render(views::login())
}

#[derive(FromForm)]
pub struct LoginData {
    pub name: String,
    pub password: String,
}

#[post("/login", data = "<data>")]
pub fn login_action(mut cookies: Cookies, mut authdb: db::AuthDB, data: Form<LoginData>) -> Result<status::Custom<String>> {
    if let Some((id, _nick)) = db::login_query(&mut *authdb, &data.name, &data.password)? {
        cookies.add_private(Cookie::new("account_id", id.to_string()));
        Ok(status::Custom(Status::Created, "logged in!".to_owned()))
    } else {
        Ok(status::Custom(Status::Forbidden, "forbidden".to_owned()))
    }
}

#[post("/logout")]
pub fn logout(mut cookies: Cookies) -> status::Custom<String> {
    cookies.remove_private(Cookie::named("account_id"));
    status::Custom(Status::Ok, "logged out!".to_owned())
}

#[get("/user/<nick>")]
pub fn user_page(user: MaybeUser, nick: String) -> Markup {
    Page::new()
        .title(&nick)
        .account(user.0.as_ref())
        .render(html! { (&nick) })
}

#[get("/campaign")]
pub fn campaign(user: MaybeUser, campaign: rocket::State<crate::system::Campaign>) -> Markup {
    Page::new()
        .title(&campaign.manifest.name)
        .account(user.0.as_ref())
        .render(views::campaign(&*campaign))
}

#[get("/characters")]
pub fn characters(user: MaybeUser) -> Markup {
    Page::new()
        .title("Персонажи")
        .account(user.0.as_ref())
        .render(views::characters())
}

#[get("/characters/new?<role>")]
pub fn new_character(user: MaybeUser, campaign: rocket::State<crate::system::Campaign>, role: Option<&RawStr>) -> Markup {
    Page::new()
        .title("Новый персонаж")
        .account(user.0.as_ref())
        .script("/static/character_form.js")
        .render(views::character_form(&*campaign, role.map(|r| r.url_decode().unwrap()).as_deref()))
}

#[get("/characters/<id>/edit")]
pub fn edit_character(id: u32) -> Markup {
    todo!()
}

#[post("/characters")]
pub fn create_character() -> Markup {
    todo!()
}

#[post("/characters/<id>")]
pub fn update_character(id: u32) -> Markup {
    todo!()
}

#[get("/characters/<id>")]
pub fn character_page(id: u32) -> Markup {
    todo!()
}

pub fn routes() -> Vec<Route> {
    routes![
        index,
        signup,
        login_page,
        login_action,
        logout,
        user_page,
        campaign,
        characters,
        new_character,
        edit_character,
        create_character,
        update_character,
        character_page,
    ]
}
