extern crate rocket;
extern crate rocket_contrib;

use models::Message;
use rocket::http::Status;
use rocket::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::status;
use rocket::response::status::NotFound;
use self::rocket::State;
use self::rocket_contrib::UUID;
use std::collections::HashMap;
use super::headers::CALLBACK_HEADER;
use super::server::PubSubServer;
use uuid::ParseError;

type Code = status::Custom<()>;

const OK: Code = status::Custom(Status::Ok, ());

lazy_static! {
    static ref NO_HEADET_ERR: String = {
        format!("HTTP request must have {} header containing Uuid of a subscriber", CALLBACK_HEADER)
    };
}

struct Headers { v: HashMap<String, String> }

impl<'a, 'r> FromRequest<'a, 'r> for Headers {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Headers, ()> {
        Outcome::Success(Headers {
            v: request.headers()
                .iter()
                .map(|h| (h.name.to_string(), h.value.to_string()))
                .collect()
        })
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello from Pub-Sub-Server!"
}

#[get("/subscribe/<topic>")]
fn subscribe<'r>(server: State<PubSubServer>, topic: String, headers: Headers)
                 -> Result<String, NotFound<&'static str>> {
    let l = headers.v.get(CALLBACK_HEADER)
        .ok_or(NotFound(NO_HEADET_ERR.as_ref()))?;

    println!("subscribing on topic {} location: {}", topic, l);
    let id = server.add_pending_subscriber(l.to_string(), topic);
    Ok(format!("{}", id))
}

#[delete("/subscribe/<id>")]
fn unsubscribe(server: State<PubSubServer>, id: UUID) -> Result<String, ParseError> {
    let uuid = *id;
    let h_uuid = format!("{}", uuid.hyphenated());
    println!("unsubscribe id {:?}", h_uuid);
    server.remove_subscriber(uuid);
    Ok(h_uuid)
}

#[head("/subscribe/<id>")]
fn touch_subscriber(server: State<PubSubServer>, id: UUID) -> Code {
    server.touch_subscriber(*id);
    OK
}

#[get("/publish/<id>")]
fn add_publisher(server: State<PubSubServer>, id: UUID) -> Result<String, ParseError> {
    let uuid = *id;
    let h_uuid = format!("{}", uuid.hyphenated());
    println!("adding publisher {}", h_uuid);
    server.add_publisher(uuid);
    Ok(h_uuid)
}

#[delete("/publish/<id>")]
fn remove_publisher(server: State<PubSubServer>, id: UUID) -> Code {
    let uuid = *id;
    server.remove_publisher(uuid);
    OK
}

#[head("/publish/<id>")]
fn touch_publisher(server: State<PubSubServer>, id: UUID) -> Result<(), NotFound<String>> {
    let uuid = *id;
    server.touch_publisher(uuid).map_err(|e| NotFound(e))
}

#[put("/publish/<topic>/<publisher>/<subject>", data = "<body>")]
fn publish(server: State<PubSubServer>, topic: String, publisher: UUID, subject: String,
           headers: Headers, body: String) //TODO:  set max body size
           -> Code {
    server.publish_message(Message { publisher: *publisher, topic, subject, headers: headers.v, body });
    OK
}

#[delete("/publish/<topic>/<publisher>/<subject>")]
fn remove(server: State<PubSubServer>, publisher: UUID, topic: String, subject: String, headers: Headers) -> Code {
    server.remove(Message {
        publisher: *publisher,
        topic,
        subject,
        headers: headers.v,
        body: ""
            .to_string(),
    });
    OK
}
