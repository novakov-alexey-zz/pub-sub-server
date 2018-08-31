use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use super::client::PubClient;
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
    id: Uuid
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
    client: PubClient,
    pending_subscribers: Arc<Mutex<HashMap<Uuid, Subscriber>>>,
    publishers: Arc<Mutex<HashMap<Uuid, Publisher>>>,
    subscribers: Arc<Mutex<HashMap<Topic, Vec<Subscriber>>>>,
    received_subs: Arc<Mutex<HashSet<Topic>>>,
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
            .for_each(|m| self.publish(&m, &s))
    }

    fn publish(&self, m: &Message, sub: &Subscriber) {
        println!("publish message: {:?} for subscriber: {:?}", &m, &sub);

        let url = format!("{}receive/{}/{}/{}", &sub.callback, &sub.topic, &m.publisher, &m.subject);
        let hrs = Self::rename_headers(&m.headers);
        let res = self.client.post(url, hrs, &m.body);

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
        self.publishers.lock().unwrap().insert(id, Publisher { id });
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
            let hrs = Self::rename_headers(&m.headers);

            match self.client.delete(url.clone(), hrs) {
                Ok(cs) => println!("removed result {}", cs),
                Err(s) => println!("problem on remove url {} for {:?}", url, s)
            }
        });
    }

    fn rename_headers(headers: &HashMap<String, String>) -> HashMap<String, String> {
        headers
            .iter()
            .map(|(k, v)| (format!("info-{}", k.to_owned()), v.to_owned()))
            .collect()
    }
}