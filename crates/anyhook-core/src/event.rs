use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub source: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub payload: Value,
    pub metadata: Option<Value>,
}

impl Event {
    pub fn new(source: impl Into<String>, event_type: impl Into<String>, payload: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            source: source.into(),
            event_type: event_type.into(),
            timestamp: Utc::now(),
            payload,
            metadata: None,
        }
    }
}
