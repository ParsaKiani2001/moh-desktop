use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Message {
    #[serde(rename = "Register")]
    Register { topics: Vec<String> },
    
    #[serde(rename = "Publish")]
    Publish { 
        topic: String, 
        payload: serde_json::Value 
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingMessage {
    pub topic: String,
    pub payload: serde_json::Value,
}

impl Message {
    pub fn register(topics: Vec<&str>) -> Self {
        Message::Register {
            topics: topics.into_iter().map(|s| s.to_string()).collect(),
        }
    }
    
    pub fn publish(topic: &str, payload: serde_json::Value) -> Self {
        Message::Publish {
            topic: topic.to_string(),
            payload,
        }
    }
    
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}