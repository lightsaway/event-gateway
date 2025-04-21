use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StringExpression {
    RegexMatch {
        #[serde(
            serialize_with = "regex_serialize",
            deserialize_with = "regex_deserialize"
        )]
        value: Regex,
    },
    Equals {
        value: String,
    },
    StartsWith {
        value: String,
    },
    EndsWith {
        value: String,
    },
    Contains {
        value: String,
    },
}

impl PartialEq for StringExpression {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StringExpression::RegexMatch { value: left }, StringExpression::RegexMatch { value: right }) => {
                left.as_str() == right.as_str()
            }
            (StringExpression::Equals { value: left }, StringExpression::Equals { value: right })
            | (StringExpression::StartsWith { value: left }, StringExpression::StartsWith { value: right })
            | (StringExpression::EndsWith { value: left }, StringExpression::EndsWith { value: right })
            | (StringExpression::Contains { value: left }, StringExpression::Contains { value: right }) => {
                left == right
            }
            _ => false,
        }
    }
}

#[derive(Clone, Serialize, PartialEq,Debug, Deserialize)]
#[serde(
    rename_all = "lowercase",
)]
pub enum Condition {
    AND(Vec<Condition>),
    OR(Vec<Condition>),
    NOT(Box<Condition>),
    ANY,
    #[serde(untagged)]
    ONE(StringExpression),
}

fn regex_serialize<S>(regex: &Regex, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(regex.as_str())
}

fn regex_deserialize<'de, D>(deserializer: D) -> Result<Regex, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)?
        .parse()
        .map_err(D::Error::custom)
}

impl Condition {
    pub fn matches(&self, to: &str) -> bool {
        match self {
            Condition::ANY => true,
            Condition::ONE(expr) => match expr {
                StringExpression::RegexMatch { value } => value.is_match(to),
                StringExpression::Equals { value } => value == to,
                StringExpression::StartsWith { value } => to.starts_with(value),
                StringExpression::EndsWith { value } => to.ends_with(value),
                StringExpression::Contains { value } => to.contains(value),
            },
            Condition::AND(conditions) => conditions.iter().all(|cond| cond.matches(to)),
            Condition::OR(conditions) => conditions.iter().any(|cond| cond.matches(to)),
            Condition::NOT(condition) => !condition.matches(to),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_any_match() {
        let condition = Condition::ANY;
        assert!(condition.matches("test123"));
        assert!(condition.matches("random"));
    }

    #[test]
    fn test_serialization_deserialization() {
        // Test ANY condition
        let any_condition = Condition::ANY;
        let serialized = serde_json::to_string(&any_condition).expect("Failed to serialize ANY condition");
        assert_eq!(serialized, r#""any""#);
        let deserialized: Condition = serde_json::from_str(&serialized).expect("Failed to deserialize ANY condition");
        assert!(matches!(deserialized, Condition::ANY));

        // Test Equals condition
        let equals_condition = Condition::ONE(StringExpression::Equals {
            value: "test".to_string(),
        });
        let serialized = serde_json::to_string(&equals_condition).expect("Failed to serialize Equals condition");
        assert_eq!(serialized, r#"{"type":"equals","value":"test"}"#);
        let deserialized: Condition = serde_json::from_str(&serialized).expect("Failed to deserialize Equals condition");
        assert!(matches!(deserialized, Condition::ONE(StringExpression::Equals { value }) if value == "test"));

        // Test RegexMatch condition
        let regex = Regex::new("^test.*").unwrap();
        let regex_condition = Condition::ONE(StringExpression::RegexMatch { value: regex });
        let serialized = serde_json::to_string(&regex_condition).expect("Failed to serialize RegexMatch condition");
        assert_eq!(serialized, r#"{"type":"regexMatch","value":"^test.*"}"#);
        let deserialized: Condition = serde_json::from_str(&serialized).expect("Failed to deserialize RegexMatch condition");
        assert!(matches!(deserialized, Condition::ONE(StringExpression::RegexMatch { value }) if value.as_str() == "^test.*"));

        // Test StartsWith condition
        let starts_with_condition = Condition::ONE(StringExpression::StartsWith {
            value: "test".to_string(),
        });
        let serialized = serde_json::to_string(&starts_with_condition).expect("Failed to serialize StartsWith condition");
        assert_eq!(serialized, r#"{"type":"startsWith","value":"test"}"#);
        let deserialized: Condition = serde_json::from_str(&serialized).expect("Failed to deserialize StartsWith condition");
        assert!(matches!(deserialized, Condition::ONE(StringExpression::StartsWith { value }) if value == "test"));

        // Test EndsWith condition
        let ends_with_condition = Condition::ONE(StringExpression::EndsWith {
            value: "test".to_string(),
        });
        let serialized = serde_json::to_string(&ends_with_condition).expect("Failed to serialize EndsWith condition");
        assert_eq!(serialized, r#"{"type":"endsWith","value":"test"}"#);
        let deserialized: Condition = serde_json::from_str(&serialized).expect("Failed to deserialize EndsWith condition");
        assert!(matches!(deserialized, Condition::ONE(StringExpression::EndsWith { value }) if value == "test"));

        // Test Contains condition
        let contains_condition = Condition::ONE(StringExpression::Contains {
            value: "test".to_string(),
        });
        let serialized = serde_json::to_string(&contains_condition).expect("Failed to serialize Contains condition");
        assert_eq!(serialized, r#"{"type":"contains","value":"test"}"#);
        let deserialized: Condition = serde_json::from_str(&serialized).expect("Failed to deserialize Contains condition");
        assert!(matches!(deserialized, Condition::ONE(StringExpression::Contains { value }) if value == "test"));

        // Test AND condition
        let and_condition = Condition::AND(vec![
            Condition::ONE(StringExpression::Equals {
                value: "test1".to_string(),
            }),
            Condition::ONE(StringExpression::Equals {
                value: "test2".to_string(),
            }),
        ]);
        let serialized = serde_json::to_string(&and_condition).expect("Failed to serialize AND condition");
        assert_eq!(serialized, r#"{"and":[{"type":"equals","value":"test1"},{"type":"equals","value":"test2"}]}"#);
        let deserialized: Condition = serde_json::from_str(&serialized).expect("Failed to deserialize AND condition");
        assert!(matches!(deserialized, Condition::AND(conditions) if conditions.len() == 2));

        // Test OR condition
        let or_condition = Condition::OR(vec![
            Condition::ONE(StringExpression::Equals {
                value: "test1".to_string(),
            }),
            Condition::ONE(StringExpression::Equals {
                value: "test2".to_string(),
            }),
        ]);
        let serialized = serde_json::to_string(&or_condition).expect("Failed to serialize OR condition");
        assert_eq!(serialized, r#"{"or":[{"type":"equals","value":"test1"},{"type":"equals","value":"test2"}]}"#);
        let deserialized: Condition = serde_json::from_str(&serialized).expect("Failed to deserialize OR condition");
        assert!(matches!(deserialized, Condition::OR(conditions) if conditions.len() == 2));

        // Test NOT condition
        let not_condition = Condition::NOT(Box::new(Condition::ONE(StringExpression::Equals {
            value: "test".to_string(),
        })));
        let serialized = serde_json::to_string(&not_condition).expect("Failed to serialize NOT condition");
        assert_eq!(serialized, r#"{"not":{"type":"equals","value":"test"}}"#);
        let deserialized: Condition = serde_json::from_str(&serialized).expect("Failed to deserialize NOT condition");
        assert!(matches!(deserialized, Condition::NOT(_)));
    }

    #[test]
    fn test_complex_condition_serialization() {
        // Test a complex condition with nested AND, OR, and NOT
        let complex_condition = Condition::AND(vec![
            Condition::OR(vec![
                Condition::ONE(StringExpression::Equals {
                    value: "test1".to_string(),
                }),
                Condition::NOT(Box::new(Condition::ONE(StringExpression::Equals {
                    value: "test2".to_string(),
                }))),
            ]),
            Condition::ONE(StringExpression::StartsWith {
                value: "prefix".to_string(),
            }),
        ]);

        let serialized = serde_json::to_string(&complex_condition).expect("Failed to serialize complex condition");
        let deserialized: Condition = serde_json::from_str(&serialized).expect("Failed to deserialize complex condition");
        
        // Verify the structure matches
        match deserialized {
            Condition::AND(conditions) => {
                assert_eq!(conditions.len(), 2);
                match &conditions[0] {
                    Condition::OR(or_conditions) => {
                        assert_eq!(or_conditions.len(), 2);
                        assert!(matches!(&or_conditions[0], Condition::ONE(StringExpression::Equals { value }) if value == "test1"));
                        assert!(matches!(&or_conditions[1], Condition::NOT(_)));
                    }
                    _ => panic!("Expected OR condition"),
                }
                assert!(matches!(&conditions[1], Condition::ONE(StringExpression::StartsWith { value }) if value == "prefix"));
            }
            _ => panic!("Expected AND condition"),
        }
    }

    #[test]
    fn test_regex_match() {
        let value = Regex::new("^test.*").unwrap();
        let condition = Condition::ONE(StringExpression::RegexMatch { value });
        assert!(condition.matches("test123"));
        assert!(!condition.matches("random"));
    }

    #[test]
    fn test_equals() {
        let condition = Condition::ONE(StringExpression::Equals {
            value: "test".to_string(),
        });
        assert!(condition.matches("test"));
        assert!(!condition.matches("Test"));
    }

    #[test]
    fn test_starts_with() {
        let condition = Condition::ONE(StringExpression::StartsWith {
            value: "start".to_string(),
        });
        assert!(condition.matches("start_here"));
        assert!(!condition.matches("finish_start"));
    }

    #[test]
    fn test_ends_with() {
        let condition = Condition::ONE(StringExpression::EndsWith {
            value: "end".to_string(),
        });
        assert!(condition.matches("the_end"));
        assert!(!condition.matches("end_the"));
    }

    #[test]
    fn test_contains() {
        let condition = Condition::ONE(StringExpression::Contains {
            value: "inside".to_string(),
        });
        assert!(condition.matches("this_is_inside_that"));
        assert!(!condition.matches("outside"));
    }

    #[test]
    fn test_and_conditions() {
        let and_condition = Condition::AND(vec![
            Condition::ONE(StringExpression::StartsWith {
                value: "start".to_string(),
            }),
            Condition::ONE(StringExpression::EndsWith {
                value: "finish".to_string(),
            }),
        ]);
        assert!(and_condition.matches("start_middle_finish"));
        assert!(!and_condition.matches("start_finish_fail"));
    }

    #[test]
    fn test_or_conditions() {
        let or_condition = Condition::OR(vec![
            Condition::ONE(StringExpression::Equals {
                value: "option1".to_string(),
            }),
            Condition::ONE(StringExpression::Equals {
                value: "option2".to_string(),
            }),
        ]);
        assert!(or_condition.matches("option1"));
        assert!(or_condition.matches("option2"));
        assert!(!or_condition.matches("option3"));
    }

    #[test]
    fn test_not_condition() {
        let not_condition = Condition::NOT(Box::new(Condition::ONE(StringExpression::Equals {
            value: "nope".to_string(),
        })));
        assert!(not_condition.matches("yes"));
        assert!(!not_condition.matches("nope"));
    }

    #[test]
    fn serialize_deserialize_condition() {
        let value = Regex::new("^test.*").unwrap();
        let condition = Condition::AND(vec![
            Condition::ONE(StringExpression::RegexMatch { value }),
            Condition::ONE(StringExpression::Equals {
                value: "test123".to_string(),
            }),
        ]);

        let serialized = serde_json::to_string(&condition).expect("Failed to serialize condition");

        let deserialized: Condition =
            serde_json::from_str(&serialized).expect("Failed to deserialize condition");

        // You might want to check deserialized conditions
        // Since you cannot directly compare `Regex`, you must test the actual matching functionality
        assert!(deserialized.matches("test123"));
        assert!(!deserialized.matches("no_match"));
        assert_eq!(
            serialized,
            r#"{"and":[{"type":"regexMatch","value":"^test.*"},{"type":"equals","value":"test123"}]}"#
        );
    }
}
