use actix_web::web;
use actix_web::web::ServiceConfig;
use async_trait::async_trait;
use event_stream::{EventMetaData, OrphanWrapper};
use event_stream::{EventStream, Handler};
use serde_json::{Value, from_str};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
mod prefs;
use crate::prefs::db::Preferences;
pub trait Sender: Send + Sync {
    fn send(&self, address: String, subject: String, message: String);
}

struct ConsoleSender;

impl Sender for ConsoleSender {
    fn send(&self, address: String, subject: String, message: String) {
        println!("message sent to {address} subject: {subject}, message: {message}")
    }
}
#[derive(Clone)]
pub struct Module {
    sender: Arc<dyn Sender>,
    state: Arc<Preferences>,
}

struct OnNotification {
    state: Arc<Preferences>,
    sender: Arc<dyn Sender>,
}

use crate::prefs::config;

#[async_trait]
impl Handler for OnNotification {
    async fn handle(&self, subject: String, message: Vec<u8>) {
        let message = String::from_utf8(message).unwrap();
        let emd = from_str::<Value>(&message).unwrap();
        let event: EventMetaData = from_str(&emd["metadata"].as_str().unwrap()).unwrap();
        let address = match self
            .state
            .get(&event.user_id.unwrap().to_string(), &subject)
            .await
        {
            Ok(r) => r.unwrap(),
            Err(e) => {
                eprintln!("Error in reading preferences: {e}");
                return;
            }
        };
        self.sender.send(address, subject, message);
    }
}

impl Module {
    pub async fn new(pool: Pool<Sqlite>, es: OrphanWrapper<Arc<dyn EventStream>>) -> Self {
        let sender: Arc<dyn Sender> = Arc::new(ConsoleSender {});
        let state = Arc::new(Preferences::new(pool.clone()));
        let module = Self {
            sender,
            state: state.clone(),
        };
        module.subscribe(es.0, state).await;
        module
    }

    pub fn with_sender(mut self, sender: Arc<dyn Sender>) -> Self {
        self.sender = sender;
        self
    }

    pub fn config(&self, cfg: &mut ServiceConfig, namespace: &str) {
        cfg.service(
            web::scope(namespace)
                .app_data(web::Data::from(self.state.clone()))
                .configure(config),
        );
    }
    pub async fn subscribe(&self, es: Arc<dyn EventStream>, state: Arc<Preferences>) {
        match es
            .clone()
            .subscribe(
                ">".to_string(),
                Arc::new(OnNotification {
                    sender: self.sender.clone(),
                    state,
                }),
            )
            .await
        {
            Ok(_) => (),
            Err(e) => eprintln!("Error in subscribing to event stream: {e}"),
        };
    }
}
