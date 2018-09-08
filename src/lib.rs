#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate chrono;
extern crate rocket;
extern crate uuid;

use rocket::Rocket;
use self::rest::*;
use server::PubSubServer;

pub mod rest;
pub mod server;
pub mod client;
mod headers;

pub fn mount_routes(server: PubSubServer) -> Rocket {
    rocket::ignite()
        .manage(server)
        .mount(
            "/info",
            routes![
                index,
                subscribe,
                unsubscribe,
                touch_subscriber,
                add_publisher,
                remove_publisher,
                touch_publisher,
                publish,
                remove
            ],
        )
}