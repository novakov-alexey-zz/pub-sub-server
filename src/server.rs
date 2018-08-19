use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug)]
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

pub struct PubSubServer {
    pending_subscribers: Arc<Mutex<HashMap<Uuid, Subscriber>>>
}

impl PubSubServer {
    pub fn new() -> Self {
        PubSubServer {
            pending_subscribers: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn add_subscriber(&self, callback: String, topic: String) -> Uuid {
        let sub = Subscriber::new(callback, topic);
        let id = sub.id.clone();
        println!("adding {:?} to pending", sub);
        self.pending_subscribers.lock().unwrap().insert(sub.id, sub);
        id
    }
}