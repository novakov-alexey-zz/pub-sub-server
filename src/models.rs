use std::collections::HashMap;
use chrono::prelude::*;
use uuid::Uuid;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Subscriber {
    pub id: Uuid,
    pub callback: String,
    pub topic: String,
}

impl Subscriber {
    pub fn new(callback: String, topic: String) -> Self {
        Subscriber { id: Uuid::new_v4(), callback, topic }
    }
}

impl Display for Subscriber {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.id.hyphenated(), self.callback, self.topic)
    }
}

#[derive(Debug)]
pub struct Publisher {
    pub id: Uuid,
    last_seen: DateTime<Local>,
}

impl Publisher {
    pub fn new(id: Uuid) -> Self {
        Publisher {
            id,
            last_seen: Local::now(),
        }
    }

    pub fn touch(&mut self) {
        self.last_seen = Local::now()
    }
}

impl Display for Publisher {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.id.hyphenated(), self.last_seen)
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub publisher: Uuid,
    pub topic: Topic,
    pub subject: Subject,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Message {
    pub fn with_headers(self, h: HashMap<String, String>) -> Message {
        Message { headers: h, ..self }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {}, {}, {:?}, \n body: {})", self.publisher.hyphenated(), self.topic, self
            .subject, self.headers, self.body)
    }
}

pub type Subject = String;
pub type Topic = String;