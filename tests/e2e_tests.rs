extern crate pub_sub_server;
extern crate rocket;

use pub_sub_server::mount_routes;
use pub_sub_server::server::PubSubServer;
use rocket::http::Header;
use rocket::http::Status;
use rocket::local::Client;

#[test]
fn subscribe() {
    //given
    let client = new_client();

    //when
    let mut res = client
        .get("info/subscribe/topic1")
        .header(Header::new("Location", "my_location"))
        .dispatch();

    //then
    assert_eq!(res.status(), Status::Ok);

    let body = res.body_string();
    assert!(body.is_some());

    let id = body.unwrap();
    println!("{}", id);
    assert_eq!(id.len(), 36);
}

#[test]
fn unsubscribe() {
    //given
    let client = new_client();
    let id = "355f2e4f-554b-47d7-aca8-122a6cec9f26";

    //when
    let mut res = client
        .delete(format!("info/subscribe/{}", id))
        .dispatch();

    //then
    assert_eq!(res.status(), Status::Ok);

    let body = res.body_string();
    assert!(body.is_some());

    let returned_id = body.unwrap();
    assert_eq!(returned_id, id);
}

#[test]
fn touch_subscriber() {
    //given
    let client = new_client();
    let mut res = client
        .get("info/subscribe/topic1")
        .header(Header::new("Location", "my_location"))
        .dispatch();
    let id = res.body_string().unwrap();

    //when
    let mut subscribed = client
        .head(format!("info/subscribe/{}", id))
        .dispatch();

    //then
    assert_eq!(subscribed.status(), Status::Ok);

    let body = subscribed.body_string();
    assert!(body.is_none());

    //when
    let removed = client
        .delete(format!("info/subscribe/{}", id))
        .dispatch();
    assert_eq!(removed.status(), Status::Ok);

    let touched = client
        .head(format!("info/subscribe/{}", id))
        .dispatch();

    assert_eq!(touched.status(), Status::Ok);
}

#[test]
fn add_publisher() {
    //given
    let client = new_client();
    let id = "355f2e4f-554b-47d7-aca8-122a6cec9f26";
    //when
    let mut res = client
        .get(format!("info/publish/{}", id))
        .dispatch();
    let res_id = res.body_string().unwrap();
    //then
    assert_eq!(id, res_id);
}

#[test]
fn touch_remove_touch_publisher() {
    //given
    let client = new_client();
    let id = "355f2e4f-554b-47d7-aca8-122a6cec9f26";

    //when
    client
        .get(format!("info/publish/{}", id))
        .dispatch();
    let mut res = client
        .head(format!("info/publish/{}", id))
        .dispatch();
    let body = res.body_string();

    //then
    assert!(body.is_none());

    //when
    let mut res = client
        .delete(format!("info/publish/{}", id))
        .dispatch();
    let body = res.body_string();

    //then
    assert!(body.is_none());

    //when
    let mut res = client
        .head(format!("info/publish/{}", id))
        .dispatch();
    let code = res.status();

    //then
    assert_eq!(404, code.code);

    let body = res.body_string();
    assert!(body.is_some());

    let body = body.unwrap();
    assert!(body.contains("Touching unknown publisher "));
}

fn new_client() -> Client {
    let rocket = mount_routes(PubSubServer::new());
    Client::new(rocket).expect("valid rocket instance")
}