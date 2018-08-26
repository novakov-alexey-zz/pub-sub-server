extern crate pub_sub_server;
extern crate rocket;

use pub_sub_server::mount_routes;
use rocket::local::Client;
use pub_sub_server::server::PubSubServer;

#[test]
fn subscribe() {
    let rocket = mount_routes(PubSubServer::new());
    let client = Client::new(rocket).expect("valid rocket instance");
}