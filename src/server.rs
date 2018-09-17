use chrono::prelude::*;
use client::Subscribers;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use super::client::SubscriberService;
use super::headers::unformat_headers;
use uuid::Uuid;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt;

#[derive(Debug, Clone)]
struct Subscriber {
    id: Uuid,
    callback: String,
    topic: String,
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
struct Publisher {
    id: Uuid,
    last_seen: DateTime<Local>,
}

impl Publisher {
    fn new(id: Uuid) -> Self {
        Publisher {
            id,
            last_seen: Local::now(),
        }
    }

    fn touch(&mut self) {
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
    fn with_headers(self, h: HashMap<String, String>) -> Message {
        Message { headers: h, ..self }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {}, {}, {:?}, \n body: {})", self.publisher.hyphenated(), self.topic, self
            .subject, self.headers, self.body)
    }
}

type Subject = String;
type Topic = String;

pub struct PubSubServer {
    pub subs_service: Box<Subscribers + 'static>,
    pending_subscribers: Arc<Mutex<HashMap<Uuid, Subscriber>>>,
    publishers: Arc<Mutex<HashMap<Uuid, Publisher>>>,
    subscribers: Arc<Mutex<HashMap<Topic, Vec<Subscriber>>>>,
    // topics - main data container. A Subject can have only one message, i.e. Subject is a
    // unique of a Message
    topics: Arc<Mutex<HashMap<Topic, HashMap<Uuid, HashMap<Subject, Message>>>>>,
}

unsafe impl<'a> Send for PubSubServer {}

unsafe impl<'a> Sync for PubSubServer {}

impl<'a> PubSubServer {
    pub fn new() -> Self {
        PubSubServer::with_service(Box::new(SubscriberService::new()))
    }

    pub fn with_service(client: Box<Subscribers + 'a>) -> PubSubServer {
        PubSubServer {
            subs_service: client,
            pending_subscribers: Arc::new(Mutex::new(HashMap::new())),
            publishers: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            topics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_pending_subscriber(&self, callback: String, topic: Topic) -> Uuid {
        let sub = Subscriber::new(callback, topic);
        let id = sub.id.clone();
        println!("adding {} to pending", sub);
        self.pending_subscribers.lock().unwrap().insert(sub.id, sub);
        id
    }

    pub fn remove_subscriber(&self, id: Uuid) {
        for (_, subs) in self.subscribers.lock().unwrap().iter_mut() {
            subs.retain(|s| s.id != id);
        }
    }

    pub fn touch_subscriber(&self, id: Uuid) {
        self.pending_subscribers.lock().unwrap().get(&id).iter()
            .for_each(|s| println!("Found subscriber {}", s));

        self.pending_subscribers.lock().unwrap()
            .remove(&id)
            .into_iter()
            .for_each(|s| self.add_subscriber(s))
    }

    fn add_subscriber(&self, s: Subscriber) {
        self.subscribers.lock().unwrap()
            .entry(s.topic.clone())
            .or_insert(vec![])
            .push(s.clone());

        self.publish_all_messages(s)
    }

    fn publish_all_messages(&self, s: Subscriber) {
        println!("publishing all message for subscriber {}", s);
        self.topics.lock().unwrap()
            .entry(s.topic.clone())
            .or_insert(HashMap::new())
            .values()
            .into_iter()
            .flat_map(|m| m.values())
            .into_iter()
            .for_each(|m| self.publish(&m, &s))
    }

    fn publish(&self, m: &Message, sub: &Subscriber) {
        println!("publish message: {} for subscriber: {}", &m, &sub);
        let msg = Message {
            publisher: m.publisher.clone(),
            topic: sub.topic.clone(),
            subject: m.subject.clone(),
            headers: m.headers.clone(),
            body: m.body.clone(),
        };

        let c = self.subs_service.as_ref();
        let res = c.publish_message(&sub.callback, &msg);

        match res {
            Ok(_) =>
                println!("message publishing for {} returned Ok", &sub),
            Err(s) => {
                self.remove_subscriber(sub.id);
                println!("message publishing failed for {} with status: {:?}", &sub, s);
            }
        }
    }

    pub fn add_publisher(&self, id: Uuid) {
        self.publishers.lock().unwrap().insert(id, Publisher::new(id));
        match self.publishers.lock().unwrap().get(&id) {
            Some(p) => println!("added publisher {}", p),
            None => println!("WARNING: publisher with id = {} is not found", id)
        }
    }

    pub fn remove_publisher(&self, id: Uuid) {
        match self.publishers.lock().unwrap()
            .remove(&id) {
            Some(p) => {
                self.remove_publisher_topics(&id);
                println!("removed publisher {}", p)
            }
            None => println!("publisher not found. Doing nothing")
        }
    }

    fn remove_publisher_topics(&self, id: &Uuid) {
        self.topics.lock().unwrap()
            .iter_mut()
            .for_each(|(_, pubs)| {
                pubs.remove(id).iter()
                    .for_each(|msgs| {
                        &msgs.iter()
                            .for_each(|(_, msg)| {
                                self.subscribers.lock().unwrap()
                                    .get(msg.topic.as_str())
                                    .iter()
                                    .for_each(|s| self.remove_message(&msg, s))
                            });
                    });
            });
    }

    fn remove_message(&self, m: &Message, subscribers: &Vec<Subscriber>) {
        subscribers.iter().for_each(|s| {
            println!("remove message for subscriber = {} on callback = {} and topic = {}", s,
                     &s.callback, &s.topic);
            let c = self.subs_service.as_ref();
            let msg = Message {
                publisher: m.publisher,
                topic: s.topic.clone(),
                subject: m.subject.clone(),
                headers: m.headers.clone(),
                body: "".to_string(),
            };

            match c.remove_message(&s.callback, &msg) {
                Ok(cs) => println!("removed result {}", cs),
                Err(e) => println!("problem on message remove callback = '{}' and topic = '{}' for \
                subscriber: '{:?}', error: {:?}", &s.callback, &s.topic, s, e)
            }
        });
    }

    pub fn touch_publisher(&self, id: Uuid) -> Result<(), String> {
        match self.publishers.lock().unwrap().get_mut(&id) {
            Some(p) => {
                println!("touching publisher {}", &p);
                p.touch();
                Ok(())
            }
            None => {
                println!("touching unknown publisher {}", id);
                Err(format!("Touching unknown publisher with id: {}", id))
            }
        }
    }

    pub fn publish_message(&self, m: Message) {
        let publisher = &m.publisher.clone();
        let headers = &m.headers.clone();
        let msg = m.with_headers(unformat_headers(headers));

        match self.publishers.lock().unwrap().get_mut(publisher) {
            Some(p) => {
                p.touch();
                self.register_message(msg.clone());
                self.fire_receive(msg);
            }
            None => println!("Ignoring unknown publisher at message: {}", &msg)
        }
    }

    fn register_message(&self, m: Message) {
        self.topics.lock().unwrap()
            .entry(m.topic.clone())
            .or_insert(HashMap::new())
            .entry(m.publisher.clone())
            .or_insert(HashMap::new())
            .insert(m.subject.clone(), m);
    }

    fn fire_receive(&self, m: Message) {
        self.subscribers.lock().unwrap()
            .entry(m.topic.clone())
            .or_insert(vec![])
            .iter()
            .for_each(|s| self.publish(&m, s))
    }

    pub fn remove(&self, m: Message) {
        self.publishers.lock().unwrap()
            .get_mut(&m.publisher)
            .iter_mut()
            .for_each(|ref mut p| {
                p.touch();
                println!("publisher remove {:?}", &m);
                self.remove_messages(&m);
                self.subscribers.lock().unwrap()
                    .get(m.topic.as_str())
                    .iter()
                    .for_each(|subs| self.remove_message(&m, subs));
            })
    }

    fn remove_messages(&self, m: &Message) {
        self.topics.lock().unwrap()
            .entry(m.topic.clone())
            .or_insert(HashMap::new())
            .entry(m.publisher)
            .or_insert(HashMap::new())
            .remove(&m.subject);
    }
}