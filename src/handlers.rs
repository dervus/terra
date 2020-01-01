use rocket::Route;
use rocket::request::Form;
use rocket::response::status;
use rocket::http::{Cookies, Cookie, Status};
use ring::digest;
use maud::{Markup, html};
use anyhow::Result;
use crate::guards::MaybeUser;
use crate::render::Page;
use crate::views;
use crate::db;

#[get("/")]
pub fn index(user: MaybeUser) -> Markup {
    Page::root()
        .account(user.0.as_ref())
        .render(html! { "Hello world!" })
}

#[get("/signup")]
pub fn signup(user: MaybeUser) -> Markup {
    Page::root()
        .title("Регистрация")
        .account(user.0.as_ref())
        .render(views::signup())
}

#[get("/login")]
pub fn login_page() -> Markup {
    Page::root()
        .title("Вход")
        .render(views::login())
}

#[derive(FromForm)]
pub struct LoginData {
    pub name: String,
    pub password: String,
}

pub fn hexstring(input: &[u8]) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(input.len() * 2);
    for byte in input.iter() {
        write!(&mut output, "{:X}", byte).unwrap();
    }
    output
}

#[post("/login", data = "<data>")]
pub fn login_action(mut cookies: Cookies, mut authdb: db::AuthDB, data: Form<LoginData>) -> Result<status::Custom<String>> {
    let result: Option<(u32, String, String)> = authdb.first_exec("SELECT id, username, sha_pass_hash FROM account WHERE UPPER(username) = UPPER(?) OR UPPER(email) = UPPER(?)", (&data.name, &data.name))?;
    if let Some((id, nick, passhash)) = result {
        if passhash.to_uppercase() == hexstring(digest::digest(&digest::SHA1, format!("{}:{}", &nick, &data.password).to_uppercase().as_bytes()).as_ref()) {
            cookies.add_private(Cookie::new("account_id", id.to_string()));
            return Ok(status::Custom(Status::Created, "logged in!".to_owned()));
        }
    }
    Ok(status::Custom(Status::Forbidden, "forbidden".to_owned()))
}

#[post("/logout")]
pub fn logout(mut cookies: Cookies) -> status::Custom<String> {
    cookies.remove_private(Cookie::named("account_id"));
    status::Custom(Status::Ok, "logged out!".to_owned())
}

#[get("/user/<nick>")]
pub fn user_page(user: MaybeUser, nick: String) -> Markup {
    Page::root()
        .title(&nick)
        .account(user.0.as_ref())
        .render(html! { (&nick) })
}

#[get("/campaigns")]
pub fn campaigns(user: MaybeUser) -> Markup {
    Page::campaigns()
        .title("Игры")
        .account(user.0.as_ref())
        .render(html! { "TODO" })
}

#[get("/characters")]
pub fn characters(user: MaybeUser, campaign: rocket::State<crate::system::Campaign>) -> Markup {
    Page::characters()
        .title("Персонажи")
        .account(user.0.as_ref())
        .script("/static/character_form.js")
        .render(views::character_form(&*campaign, None))
}

pub fn routes() -> Vec<Route> {
    routes![index, signup, login_page, login_action, logout, user_page, campaigns, characters]
}
