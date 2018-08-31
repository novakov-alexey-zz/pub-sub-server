extern crate rocket;

use rocket::http::HeaderMap;
use rocket::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::status::NotFound;
use self::rocket::State;
use std::iter::Map;
use super::server::PubSubServer;
use uuid::ParseError;
use uuid::Uuid;

pub struct InfoMessage {
    pub publisher_uuid: String,
    pub topic: String,
    pub subject: String,
    pub headers: Map<String, String>,
    pub body: String,
}

struct Headers<'a> { v: HeaderMap<'a> }

impl<'a, 'r> FromRequest<'a, 'r> for Headers<'a> {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Headers<'a>, ()> {
        Outcome::Success(Headers { v: request.headers().clone() })
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello from Pub-Sub-Server!"
}

#[get("/subscribe/<topic>")]
fn subscribe<'r>(server: State<PubSubServer>, topic: String, headers: Headers)
                 -> Result<String, NotFound<&'static str>> {
    let l = headers.v.get("Location").next()
        .ok_or(
            NotFound("HTTP request must have 'Location' header containing Uuid of a subscriber")
        )?;

    println!("subscribing on topic {} location: {}", topic, l);
    let id = server.add_pending_subscriber(l.to_string(), topic);
    Ok(format!("{}", id))
}

#[delete("/subscribe/<id>")]
fn unsubscribe(server: State<PubSubServer>, id: String) -> Result<String, ParseError> {
    let uuid = Uuid::parse_str(id.as_str())?;
    println!("unsubscribe id {}", id);
    server.remove_subscriber(uuid);
    Ok(id)
}

#[head("/subscribe/<id>")]
fn touch_subscriber(server: State<PubSubServer>, id: String) -> Result<(), ParseError> {
    let uuid = Uuid::parse_str(id.as_str())?;
    server.touch_subscriber(uuid);
    Ok(())
}

#[get("/publish/<id>")]
fn add_publisher(server: State<PubSubServer>, id: String) -> Result<String, ParseError> {
    let uuid = Uuid::parse_str(id.as_str())?;
    println!("adding publisher {}", uuid);
    server.add_publisher(uuid);
    Ok(id)
}

#[delete("/publish/<id>")]
fn remove_publisher(server: State<PubSubServer>, id: String) -> Result<(), ParseError> {
    let uuid = Uuid::parse_str(id.as_str())?;
    server.remove_publisher(uuid);
    Ok(())
}
