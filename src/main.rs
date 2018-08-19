#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate pub_sub_server;
extern crate rocket;

use pub_sub_server::rest::*;
use pub_sub_server::server::PubSubServer;

fn main() {
    rocket::ignite()
        .manage(PubSubServer::new())
        .mount(
            "/info",
            routes![
                index
            ],
        ).launch();
}
