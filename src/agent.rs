use uuid::Uuid;
use std::sync::Arc;
use crate::producer::Producer;

pub struct Agent {
    id: Uuid,
    topic: String,
    producer: Arc<dyn Producer>,
}

impl Agent {
    pub fn new(topic: String, producer: Arc<dyn Producer>) -> Self {
        Self {
            id: Uuid::new_v4(),
            topic,
            producer,
        }
    }

}
