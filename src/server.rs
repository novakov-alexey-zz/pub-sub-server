use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
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

    pub fn receive(&self, m: &Message) {
        //TODO:
    }
}

#[derive(Debug)]
struct Message {
    publisher: Uuid,
    topic: Topic,
    subject: Subject,
    headers: HashMap<String, String>,
    body: String,
}

type Subject = String;
type Topic = String;

pub struct PubSubServer {
    pending_subscribers: Arc<Mutex<HashMap<Uuid, Subscriber>>>,
    subscribers: Arc<Mutex<HashMap<String, Vec<Subscriber>>>>,
    received_subs: Arc<Mutex<HashSet<String>>>,
    topics: Arc<Mutex<HashMap<Topic, HashMap<Uuid, HashMap<Subject, Message>>>>>,
}

impl PubSubServer {
    pub fn new() -> Self {
        PubSubServer {
            pending_subscribers: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            received_subs: Arc::new(Mutex::new(HashSet::new())),
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

        self.received_subs.lock().unwrap()
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
            .for_each(|m| s.receive(m))
    }
}