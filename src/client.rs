extern crate rocket;

use rocket::http::Header;
use rocket::http::Method;
use rocket::http::Status;
use rocket::local::Client;
use std::collections::HashMap;
use super::headers::format_headers;

pub struct PubClient {
    client: Client,
}

type CodeReason<'a> = (u16, &'a str);

impl PubClient {
    pub fn new() -> Self {
        PubClient {
            client: Client::new(rocket::ignite()).expect("valid rocket")
        }
    }

    pub fn post(&self, url: String, headers: &HashMap<String, String>, body: &String)
                -> Result<&str, CodeReason> {
        self.call(Method::Post, url, headers, Some(body))
    }

    pub fn delete(&self, url: String, headers: &HashMap<String, String>) -> Result<&str, CodeReason> {
        self.call(Method::Delete, url, headers, None)
    }

    fn call(&self, method: Method, url: String, headers: &HashMap<String, String>,
            body: Option<&String>) -> Result<&str, CodeReason> {
        let mut req = match method {
            Method::Post => Ok(self.client.post(url)),
            Method::Delete => Ok(self.client.delete(url)),
            _ => Err((405u16, "unsupported http method"))
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