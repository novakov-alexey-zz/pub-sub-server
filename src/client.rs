extern crate rocket;

use rocket::http::Header;
use rocket::http::Status;
use rocket::local::Client;
use std::collections::HashMap;

pub struct PubClient {
    client: Client,
}

impl PubClient {
    pub fn new() -> Self {
        PubClient {
            client: Client::new(rocket::ignite()).expect("valid rocket")
        }
    }

    pub fn post(&self, url: String, headers: HashMap<String, String>, body: &String) -> Result<(), u16> {
        let mut req = self.client.post(url);

        for (k, v) in headers {
            req.add_header(Header::new(k, v));
        }

        let res = req.body(body).dispatch();

        match res.status() {
            Status::Ok => Ok(()),
            s => Err(s.code)
        }
    }
}