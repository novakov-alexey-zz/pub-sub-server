#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate pub_sub_server;
extern crate rocket;

use pub_sub_server::mount_routes;
use pub_sub_server::server::PubSubServer;

fn main() {
    let error = mount_routes(PubSubServer::new()).launch();
    drop(error);
}
