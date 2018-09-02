extern crate rocket;

use rocket::http::HeaderMap;
use rocket::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::status::NotFound;
use self::rocket::State;
use server::Message;
use super::server::PubSubServer;
use uuid::ParseError;
use uuid::Uuid;

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

#[head("/publish/<id>")]
fn touch_publisher(server: State<PubSubServer>, id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(id.as_str()).map_err(|e| format!("{}", e))?;
    server.touch_publisher(uuid)
}

#[put("/publish/<topic>/<publisher>/<subject>", data = "<body>")]
fn publish(server: State<PubSubServer>, topic: String, publisher: String, subject: String,
           headers: Headers, body: String) //TODO:  set max body size
           -> Result<(), ParseError> {
    let uuid = Uuid::parse_str(publisher.as_str())?;
    let map = headers.v.iter()
        .map(|h| (h.name.to_string(), h.value.to_string()))
        .collect();
    server.publish_message(Message { publisher: uuid, topic, subject, headers: map, body });
    Ok(())
}

//DELETE /info/publish/:topic/:publisher/:subject           modules.PlayInfoServerController.remove(publisher:java.util.UUID, topic:String, subject:String)

