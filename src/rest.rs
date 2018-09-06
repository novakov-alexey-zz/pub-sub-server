extern crate rocket;
extern crate rocket_contrib;

use rocket::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::status::NotFound;
use self::rocket::State;
use self::rocket_contrib::UUID;
use server::Message;
use std::collections::HashMap;
use super::server::PubSubServer;
use uuid::ParseError;
use uuid::Uuid;

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
    let l = headers.v.get("Location")
        .ok_or(
            NotFound("HTTP request must have 'Location' header containing Uuid of a subscriber")
        )?;

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
fn touch_subscriber(server: State<PubSubServer>, id: UUID) -> Result<(), ParseError> {
    server.touch_subscriber(*id);
    Ok(())
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
fn remove_publisher(server: State<PubSubServer>, id: UUID) -> Result<(), ParseError> {
    let uuid = *id;
    server.remove_publisher(uuid);
    Ok(())
}

#[head("/publish/<id>")]
fn touch_publisher(server: State<PubSubServer>, id: UUID) -> Result<(), String> {
    let uuid = *id;
    server.touch_publisher(uuid)
}

#[put("/publish/<topic>/<publisher>/<subject>", data = "<body>")]
fn publish(server: State<PubSubServer>, topic: String, publisher: String, subject: String,
           headers: Headers, body: String) //TODO:  set max body size
           -> Result<(), ParseError> {
    let uuid = Uuid::parse_str(publisher.as_str())?;
    server.publish_message(Message { publisher: uuid, topic, subject, headers: headers.v, body });
    Ok(())
}

//DELETE /info/publish/:topic/:publisher/:subject           modules.PlayInfoServerController.remove(publisher:java.util.UUID, topic:String, subject:String)

