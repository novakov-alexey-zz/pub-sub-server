extern crate rocket;

use rocket::http::HeaderMap;
use rocket::Outcome;
use rocket::request::{self, FromRequest, Request};
use self::rocket::State;
use std::iter::Map;
use super::server::PubSubServer;

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
fn subscribe<'r>(server: State<PubSubServer>, topic: String, headers: Headers) -> Result<String, String> {
    let l = headers.v.get("Location").next()
        .ok_or("HTTP Request must have 'Location' header with Uuid of the subscriber".to_string())?;
    println!("subscribing on topic {} location: {}", topic, l);
    let id = server.add_subscriber(l.to_string(), topic);
    Ok(format!("{}", id))
}
