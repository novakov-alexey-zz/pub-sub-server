extern crate rocket;

use rocket::http::Header;
use rocket::http::Method;
use rocket::http::Status;
use rocket::local::Client;
use std::collections::HashMap;
use super::headers::format_headers;
use downcast_rs::Downcast;
use models::Message;

pub trait Subscribers: Downcast {
    fn publish_message(&self, callback: &String, msg: &Message) -> Result<&str, CodeReason>;
    //TODO: return type must be Future of Result

    fn remove_message(&self, callback: &String, msg: &Message) -> Result<&str, CodeReason>;
    //TODO: return type must be Future of Result
}

impl_downcast!(Subscribers);

pub struct SubscriberService {
    client: Client,
}

pub type CodeReason<'a> = (u16, &'a str);

impl Subscribers for SubscriberService {
    fn publish_message(&self, callback: &String, msg: &Message) -> Result<&str, CodeReason> {
        let url = format!("{}receive/{}/{}/{}", callback, msg.topic, msg.publisher, msg.subject);
        self.call(Method::Post, url, &msg.headers, Some(&msg.body))
    }

    fn remove_message(&self, callback: &String, msg: &Message) -> Result<&str, CodeReason> {
        let url = format!("{}remove/{}/{}/{}", callback, msg.topic, msg.publisher, msg.subject);
        self.call(Method::Delete, url, &msg.headers, None)
    }
}

impl SubscriberService {
    pub fn new() -> Self {
        SubscriberService {
            client: Client::new(rocket::ignite()).expect("valid rocket")
        }
    }

    fn call(&self, method: Method, url: String, headers: &HashMap<String, String>,
            body: Option<&String>) -> Result<&str, CodeReason> {
        let mut req = match method {
            Method::Post => Ok(self.client.post(url)),
            Method::Delete => Ok(self.client.delete(url)),
            _ => Err((405u16, ("unsupported HTTP method")))
        }?;

        let hrs = format_headers(&headers);
        for (k, v) in hrs {
            req.add_header(Header::new(k, v));
        }

        let res = match body {
            Some(b) => req.body(b),
            None => req
        }.dispatch();

        match res.status() {
            Status::Ok => Ok(res.status().reason),
            s => Err((s.code, s.reason))
        }
    }
}