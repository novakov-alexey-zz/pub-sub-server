#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate uuid;

use rocket::Rocket;
use server::PubSubServer;
use self::rest::*;

pub mod rest;
pub mod server;

pub fn mount_routes(server: PubSubServer) -> Rocket {
    rocket::ignite()
        .manage(server)
        .mount(
            "/info",
            routes![
                index,
                subscribe
            ],
        )
}