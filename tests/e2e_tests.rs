extern crate pub_sub_server;
extern crate rocket;
extern crate uuid;

use pub_sub_server::subscribers::CodeReason;
use pub_sub_server::subscribers::Subscribers;
use pub_sub_server::mount_routes;
use pub_sub_server::server::PubSubServer;
use rocket::http::Header;
use rocket::http::Status;
use rocket::local::Client;
use std::sync::RwLock;
use pub_sub_server::models::Message;

const TOPIC_NAME: &str = "mytopic";
const SUBJECT_NAME: &str = "mysubject";
const MSG_BODY: &str = "test body";

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
    //when
    let mut subscribed = client
        .get("info/subscribe/topic1")
        .header(Header::new("Location", "my_location"))
        .dispatch();
    let id = subscribed.body_string().unwrap();

    let mut touched = client
        .head(format!("info/subscribe/{}", id))
        .dispatch();

    //then
    assert_eq!(touched.status(), Status::Ok);
    let body = touched.body_string();
    assert!(body.is_none());

    //when
    let removed = client
        .delete(format!("info/subscribe/{}", id))
        .dispatch();
    //then
    assert_eq!(removed.status(), Status::Ok);

    //when
    let touched = client
        .head(format!("info/subscribe/{}", id))
        .dispatch();
    //then
    assert_eq!(touched.status(), Status::Ok);
}

#[test]
fn add_publisher() {
    //given
    let client = new_client();
    let id = "355f2e4f-554b-47d7-aca8-122a6cec9f26";
    //when
    create_publisher(&client, id);
}

fn create_publisher(client: &Client, id: &str) {
    let mut added = client
        .get(format!("info/publish/{}", id))
        .dispatch();
    let res_id = added.body_string().unwrap();
    //then
    assert_eq!(id, res_id)
}

#[test]
fn touch_remove_touch_publisher() {
    //given
    let client = new_client();
    let id = "355f2e4f-554b-47d7-aca8-122a6cec9f26";

    //when
    let mut added = client
        .get(format!("info/publish/{}", id)) // add_publisher
        .dispatch();
    let res_id = added.body_string().unwrap();
    //then
    assert_eq!(id, res_id);

    let mut touched = client
        .head(format!("info/publish/{}", id))
        .dispatch();
    let body = touched.body_string();

    //then
    assert!(body.is_none());

    //when
    remove_publisher(&client, id);

    let mut touched = client
        .head(format!("info/publish/{}", id))
        .dispatch();
    let code = touched.status();

    //then
    assert_eq!(404, code.code);

    let body = touched.body_string();
    assert!(body.is_some());

    let text = body.unwrap();
    assert!(text.contains("Touching unknown publisher "));
}

fn remove_publisher(client: &Client, id: &str) {
    let mut removed = client
        .delete(format!("info/publish/{}", id))
        .dispatch();
    let body = removed.body_string();

    //then
    assert!(body.is_none());
}

#[test]
fn publish() {
    //given
    let client = new_client();
    let id = "355f2e4f-554b-47d7-aca8-122a6cec9f26";

    //when
    publish_message(&client, id);
}

fn publish_message(client: &Client, id: &str) {
    let res = client.put(format!("info/publish/{}/{}/{}", TOPIC_NAME, id, SUBJECT_NAME))
        .body(MSG_BODY)
        .dispatch();
    let code = res.status();
    //then
    assert_eq!(200, code.code)
}

#[test]
fn remove() {
    //given
    let client = new_client();
    let id = "355f2e4f-554b-47d7-aca8-122a6cec9f26";

    //when
    let res = client
        .delete(format!("info/publish/{}/{}/{}", TOPIC_NAME, id, SUBJECT_NAME))
        .dispatch();
    let code = res.status();

    //then
    assert_eq!(200, code.code);
}

#[test]
fn publish_subscriber_scenario() {
    //given
    let publisher_id = "8dbdd47c-cb61-44b2-8919-bd44a87fcd48";
    let client = new_client();
    //when
    create_publisher(&client, publisher_id);
    publish_message(&client, publisher_id);
    let location = "http://subscriber1:9000";

    let mut subscribed = client
        .get(format!("info/subscribe/{}", TOPIC_NAME))
        .header(Header::new("Location", location))
        .dispatch();
    let subscriber_id = subscribed.body_string().unwrap();

    let touched = client
        .head(format!("info/subscribe/{}", subscriber_id))
        .dispatch();

    //then
    assert_eq!(touched.status(), Status::Ok);

    let mock = get_mock(&client);
    let published = mock.pub_vec.read().unwrap();
    assert_eq!(published.len(), 1);

    let (callback, msg) = &published[0];
    assert_eq!(&location, callback);
    assert_eq!(TOPIC_NAME, msg.topic);
    assert_eq!(SUBJECT_NAME, msg.subject);
    assert_eq!(MSG_BODY, msg.body);

    //when
    remove_publisher(&client, &publisher_id);
    //then
    let removed = mock.remove_vec.read().unwrap();
    assert_eq!(removed.len(), 1);

    let (callback, msg) = &removed[0];
    assert_eq!(&location, callback);
    assert_eq!(TOPIC_NAME, msg.topic);
    assert_eq!(SUBJECT_NAME, msg.subject);
    assert_eq!("", msg.body);
}

fn get_mock(client: &Client) -> &MockSubscribers {
    let server: &PubSubServer = client.rocket().state().unwrap();
    let mock = server.subs_service.downcast_ref::<MockSubscribers>();
    assert_eq!(mock.is_some(), true, "Failed to downcast");
    mock.unwrap()
}

fn new_client() -> Client {
    let server = PubSubServer::with_service(Box::new(
        MockSubscribers { pub_vec: RwLock::new(Vec::new()), remove_vec: RwLock::new(Vec::new()) })
    );
    let rocket = mount_routes(server);
    Client::new(rocket).expect("valid rocket instance")
}

struct MockSubscribers {
    pub_vec: RwLock<Vec<(String, Message)>>,
    remove_vec: RwLock<Vec<(String, Message)>>,
}

impl Subscribers for MockSubscribers {
    fn publish_message(&self, callback: &String, msg: &Message) -> Result<&str, CodeReason> {
        println!("test publish_message ==== ");
        self.pub_vec.write().unwrap().push((callback.clone(), msg.clone()));
        Ok("ok")
    }

    fn remove_message(&self, callback: &String, msg: &Message) -> Result<&str, CodeReason> {
        println!("test remove_message ==== ");
        self.remove_vec.write().unwrap().push((callback.clone(), msg.clone()));
        Ok("ok")
    }
}