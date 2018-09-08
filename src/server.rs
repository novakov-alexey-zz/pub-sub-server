use chrono::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use super::client::PubClient;
use super::headers::unformat_headers;
use uuid::Uuid;

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

#[derive(Debug, Clone)]
pub struct Message {
    pub publisher: Uuid,
    pub topic: Topic,
    pub subject: Subject,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Message {
    fn set_headers(&mut self, h: HashMap<String, String>) {
        self.headers = h;
    }
}

type Subject = String;
type Topic = String;

pub struct PubSubServer {
    client: PubClient,
    pending_subscribers: Arc<Mutex<HashMap<Uuid, Subscriber>>>,
    publishers: Arc<Mutex<HashMap<Uuid, Publisher>>>,
    subscribers: Arc<Mutex<HashMap<Topic, Vec<Subscriber>>>>,
    received_topics: Arc<Mutex<HashSet<Topic>>>,
    //TODO: why received_subs set is needed at all?
    topics: Arc<Mutex<HashMap<Topic, HashMap<Uuid, HashMap<Subject, Message>>>>>,
}

impl PubSubServer {
    pub fn new() -> Self {
        PubSubServer {
            client: PubClient::new(),
            pending_subscribers: Arc::new(Mutex::new(HashMap::new())),
            publishers: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            received_topics: Arc::new(Mutex::new(HashSet::new())),
            topics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_pending_subscriber(&self, callback: String, topic: Topic) -> Uuid {
        let sub = Subscriber::new(callback, topic);
        let id = sub.id.clone();
        println!("adding {:?} to pending", sub);
        self.pending_subscribers.lock().unwrap().insert(sub.id, sub);
        id
    }

    pub fn remove_subscriber(&self, id: Uuid) {
        for (_, subs) in self.subscribers.lock().unwrap().iter_mut() {
            subs.retain(|s| s.id != id);
        }
    }

    pub fn touch_subscriber(&self, id: Uuid) {
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

        self.received_topics.lock().unwrap()
            .insert(s.topic.clone());

        self.publish_all_messages(s)
    }

    fn publish_all_messages(&self, s: Subscriber) {
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
        println!("publish message: {:?} for subscriber: {:?}", &m, &sub);

        let url = format!("{}receive/{}/{}/{}", &sub.callback, &sub.topic, &m.publisher, &m.subject);
        let res = self.client.post(url, &m.headers, &m.body);

        match res {
            Ok(_) =>
                println!("message publishing for {:?} returned Ok", &sub),
            Err(s) => {
                self.remove_subscriber(sub.id);
                println!("message publishing failed for {:?} with status: {:?}", &sub, s);
            }
        }
    }

    pub fn add_publisher(&self, id: Uuid) {
        self.publishers.lock().unwrap().insert(id, Publisher::new(id));
        println!("added publisher {:?}", self.publishers.lock().unwrap().get(&id));
    }

    pub fn remove_publisher(&self, id: Uuid) {
        match self.publishers.lock().unwrap()
            .remove(&id) {
            Some(p) => {
                self.remove_publisher_topics(&id);
                println!("removed publisher {:?}", p)
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
            let url = format!("{}remove/{}/{}/{}", s.callback, s.topic, m.publisher, m.subject);
            println!("remove message for subscriber {:?} on {}", m, url);

            match self.client.delete(url.clone(), &m.headers) {
                Ok(cs) => println!("removed result {}", cs),
                Err(s) => println!("problem on message remove url {} for {:?}", url, s)
            }
        });
    }

    pub fn touch_publisher(&self, id: Uuid) -> Result<(), String> {
        match self.publishers.lock().unwrap().get_mut(&id) {
            Some(p) => {
                println!("touching publisher {:?}", &p);
                p.touch();
                Ok(())
            }
            None => Err(format!("Touching unknown publisher with id: {}", id))
        }
    }

    pub fn publish_message(&self, mut m: Message) {
        let unformated = unformat_headers(&m.headers);
        m.set_headers(unformated);
        match self.publishers.lock().unwrap().get_mut(&m.publisher) {
            Some(p) => {
                p.touch();
                self.register_message(m.clone());
                self.register_topic(m.topic.clone());
                self.fire_receive(m);
            }
            None => println!("Ignoring unknown publisher: {:?}", &m)
        }
    }

    fn register_topic(&self, t: Topic) {
        self.received_topics.lock().unwrap().insert(t);
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