#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

mod util;
mod system;
mod render;
mod views;
mod handlers;
mod db;
mod guards;

use rocket_contrib::helmet::SpaceHelmet;
use rocket_contrib::serve::StaticFiles;

fn main() {
    let shared_system = system::load_shared_system("/home/me/code/terra-system").unwrap();
    let campaign = system::load_campaign("/home/me/code/terra-system", "last-bastion", Some(&shared_system)).unwrap();

    rocket::ignite()
        .manage(campaign)
        .attach(SpaceHelmet::default())
        .attach(db::AuthDB::fairing())
        .attach(db::CharsDB::fairing())
        .mount("/static", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")))
        .mount("/system/assets", StaticFiles::from("/home/me/code/terra-system/public"))
        .mount("/campaigns/last-bastion/assets", StaticFiles::from("/home/me/code/terra-system/campaigns/last-bastion/assets"))
        .mount("/", handlers::routes())
        .launch();
}
