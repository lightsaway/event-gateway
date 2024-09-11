use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Serialize, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Json,
    String,
    Binary,
}
#[derive(Clone, Serialize, Debug, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase", content = "content")]
pub enum Data {
    Json(HashMap<String, serde_json::Value>),
    String(String),
    Binary(Vec<u8>),
}

#[derive(Clone, Serialize, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub event_version: Option<String>,
    pub metadata: HashMap<String, String>,
    pub transport_metadata: Option<HashMap<String, String>>,
    pub data_type: Option<DataType>,
    pub data: Data,
    pub timestamp: Option<DateTime<Utc>>,
    pub origin: Option<String>,
}

impl Event {
    // fn random() -> Event {
    //     // let metadata = [
    //     //     ("key1".to_string(), "value1".to_string()),
    //     //     ("key2".to_string(), "value2".to_string()),
    //     // ]
    //     // .iter()
    //     // .cloned()
    //     // .collect();

    //     // let json_data = json!({
    //     //     "field1": "value1",
    //     //     "field2": 12345,
    //     // });

    //     // // Assuming we want to generate random data for the `Json` variant of `Data`.
    //     // let data = Data::Json(json_data.as_object().unwrap().clone());

    //     // Event {
    //     //     id: Uuid::new_v4(),
    //     //     event_type: "exampleType".to_string(),
    //     //     event_version: Some("v1".to_string()),
    //     //     metadata,
    //     //     data_type: Some(DataType::Json),
    //     //     data,
    //     //     timestamp: Some(Utc::now()),
    //     //     origin: Some("exampleOrigin".to_string()),
    //     // }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serde_data_type() {
        let data_type = DataType::Json;
        let serialized = serde_json::to_string(&data_type).expect("Failed to serialize");
        assert_eq!(serialized, "\"json\"");

        let deserialized: DataType =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(deserialized, data_type);
    }

    #[test]
    fn test_serde() {
        let uuid = Uuid::new_v4();
        let timestamp = Utc::now();

        let data_json = json!({
            "key": "value"
        });

        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), "Alice".to_string());

        let event = Event {
            id: uuid,
            event_type: "test_type".to_string(),
            event_version: Some("1.0".to_string()),
            metadata,
            data_type: Some(DataType::Json),
            data: Data::Json(data_json.as_object().unwrap().clone().into_iter().collect()),
            timestamp: Some(timestamp),
            origin: Some("example".to_string()),
        };

        let serialized = serde_json::to_string(&event).expect("Failed to serialize");

        // Check a few serialized properties
        assert!(serialized.contains(&uuid.to_string()));
        assert!(serialized.contains("\"test_type\""));
        assert!(serialized.contains("\"author\":\"Alice\""));
        let deserialized: Event = serde_json::from_str(&serialized).expect("Failed to deserialize");
        println!("{}", serialized);
        assert_eq!(deserialized, event);
    }
}
