use std::{collections::HashMap, fmt};

use super::{event::DataType, expressions::Condition, topic::Topic};
use jsonschema::{Draft, JSONSchema};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Serialize, Debug, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", content = "data")]
pub enum Schema {
    Json(JSchema),
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub instance_path: String,
    pub schema_path: String,
}

impl Schema {
    pub fn is_valid(&self, data: &Value) -> bool {
        match self {
            Schema::Json(schema) => schema.compiled_schema.is_valid(data),
        }
    }

    pub fn validate(&self, data: &Value) -> Result<(), Vec<ValidationError>> {
        match self {
            Schema::Json(schema) => schema.validate(data),
        }
    }
}

pub struct JSchema {
    compiled_schema: JSONSchema,
    raw_schema: Value,
    draft_version: Draft,
}

impl fmt::Debug for JSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JSchema")
            .field("raw_schema", &self.raw_schema)
            .finish()
    }
}

impl PartialEq for JSchema {
    fn eq(&self, other: &Self) -> bool {
        self.raw_schema == other.raw_schema
    }
}

impl Clone for JSchema {
    fn clone(&self) -> Self {
        // Recompile the `JSONSchema` from the stored raw schema
        let compiled_schema = JSONSchema::compile(&self.raw_schema)
            .expect("Failed to compile the schema during cloning");

        // Clone the raw schema which is just a `serde_json::Value`
        let raw_schema = self.raw_schema.clone();
        let draft_version = self.draft_version.clone();

        // Create a new `JSchema` with the recompiled schema and cloned raw schema
        JSchema {
            compiled_schema,
            raw_schema,
            draft_version,
        }
    }
}

impl<'de> Deserialize<'de> for JSchema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn parse_draft_version(value: &Value) -> Draft {
            value.get("$schema").and_then(Value::as_str).map_or_else(
                || jsonschema::Draft::Draft7, // Default to Draft7 if not specified
                |uri| match uri {
                    "http://json-schema.org/draft-07/schema#" => jsonschema::Draft::Draft7,
                    "http://json-schema.org/draft-06/schema#" => jsonschema::Draft::Draft6,
                    "http://json-schema.org/draft-04/schema#" => jsonschema::Draft::Draft4,
                    _ => jsonschema::Draft::Draft7, // Default to Draft7 if unrecognized
                },
            )
        }

        // Deserialize the JSON Schema into a serde_json `Value`.
        let raw_schema = Value::deserialize(deserializer)?;
        let draft_version = parse_draft_version(&raw_schema);
        // Compile the `Value` into a `JSONSchema`.
        let compiled_schema = JSONSchema::options()
            .with_draft(draft_version) // Choose appropriate draft
            .compile(&raw_schema)
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;
        Ok(JSchema {
            compiled_schema,
            raw_schema,
            draft_version,
        })
    }
}

impl Serialize for JSchema {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.raw_schema.serialize(serializer)
    }
}

impl JSchema {
    pub fn validate(&self, data: &Value) -> Result<(), Vec<ValidationError>> {
        match self.compiled_schema.validate(data) {
            Ok(_) => Ok(()),
            Err(errors) => {
                let validation_errors: Vec<ValidationError> = errors
                    .map(|error| ValidationError {
                        message: error.to_string(),
                        instance_path: error.instance_path.to_string(),
                        schema_path: error.schema_path.to_string(),
                    })
                    .collect();
                Err(validation_errors)
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub struct DataSchema {
    pub name: String,
    pub description: Option<String>,
    pub schema: Schema,
    pub event_type: String,
    pub event_version: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Clone, Serialize, PartialEq, Deserialize)]
pub struct TopicValidationConfig {
    pub id: Uuid,
    pub topic: Topic,
    pub schema: DataSchema,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicRoutingRule {
    pub id: Uuid,
    pub order: i32,
    pub topic: Topic,
    pub event_type_condition: Condition,
    pub event_version_condition: Option<Condition>,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::model::expressions::StringExpression;

    use super::*;
    use serde_json;

    #[test]
    fn test_topic_routing_rule_serde() {
        let rule = TopicRoutingRule {
            id: Uuid::new_v4(),
            order: 1,
            topic: Topic::new("example").unwrap(),
            event_type_condition: Condition::ONE(StringExpression::StartsWith {
                value: "test".into(),
            }),
            event_version_condition: Some(Condition::ONE(StringExpression::Equals {
                value: "1".into(),
            })),
            description: Some("A routing rule.".into()),
        };

        let serialized = serde_json::to_string(&rule).unwrap();
        let deserialized: TopicRoutingRule = serde_json::from_str(&serialized).unwrap();
        assert_eq!(rule, deserialized);
    }

    #[test]
    fn test_data_schema_serde() {
        let raw_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string"
                }
            },
            "required": ["name"]
        });

        let schema = DataSchema {
            name: "example".into(),
            description: Some("A schema.".into()),
            schema: Schema::Json(JSchema {
                compiled_schema: JSONSchema::compile(&raw_schema).unwrap(),
                raw_schema: raw_schema,
                draft_version: Draft::Draft7,
            }),
            event_type: "example".into(),
            event_version: Some("1".into()),
            metadata: Some(HashMap::new()),
        };

        let serialized = serde_json::to_string(&schema).unwrap();
        print!("{}", serialized);
        let deserialized: DataSchema = serde_json::from_str(&serialized).unwrap();
        assert_eq!(schema, deserialized);
    }

    #[test]
    fn test_detailed_schema_validation() {
        let raw_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string"
                },
                "age": {
                    "type": "integer",
                    "minimum": 0
                }
            },
            "required": ["name"]
        });

        let schema = Schema::Json(JSchema {
            compiled_schema: JSONSchema::compile(&raw_schema).unwrap(),
            raw_schema: raw_schema,
            draft_version: Draft::Draft7,
        });

        // Test valid data
        let valid_data = serde_json::json!({
            "name": "John",
            "age": 30
        });
        assert!(schema.validate(&valid_data).is_ok());

        // Test invalid data (missing required field)
        let invalid_data = serde_json::json!({
            "age": 30
        });
        let result = schema.validate(&invalid_data);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("required"));

        // Test invalid data (wrong type)
        let invalid_type_data = serde_json::json!({
            "name": "John",
            "age": "thirty"
        });
        let result = schema.validate(&invalid_type_data);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors[0].instance_path.contains("age"));
    }
}
