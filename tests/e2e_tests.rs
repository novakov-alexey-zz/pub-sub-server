extern crate pub_sub_server;
extern crate rocket;

use pub_sub_server::mount_routes;
use pub_sub_server::server::PubSubServer;
use rocket::http::Header;
use rocket::http::Status;
use rocket::local::Client;

#[test]
fn subscribe() {
    // given
    let rocket = mount_routes(PubSubServer::new());
    let client = Client::new(rocket).expect("valid rocket instance");

    //when
    let mut res = client
        .get("info//subscribe/topic1")
        .header(Header::new("Location", "my_location"))
        .dispatch();

    //then
    assert_eq!(res.status(), Status::Ok);

    let body = res.body_string();
    assert!(body.is_some());

    let id = body.unwrap();
    assert_eq!(id.len(), 36);
}