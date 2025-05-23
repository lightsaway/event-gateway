use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Topic(String);

#[derive(Debug, Clone)]
pub enum TopicValidationError {
    Empty,
    InvalidCharacters(String),
    TooLong(usize),
}

impl fmt::Display for TopicValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopicValidationError::Empty => write!(f, "Topic cannot be empty"),
            TopicValidationError::InvalidCharacters(chars) => {
                write!(f, "Topic contains invalid characters: {}", chars)
            }
            TopicValidationError::TooLong(len) => {
                write!(f, "Topic is too long: {} characters (max: 255)", len)
            }
        }
    }
}

impl std::error::Error for TopicValidationError {}

impl Topic {
    /// Create a new Topic with validation
    /// Topics must:
    /// - Not be empty
    /// - Contain only alphanumeric characters, dots, hyphens, and underscores
    /// - Be no longer than 255 characters
    pub fn new(s: impl Into<String>) -> Result<Self, TopicValidationError> {
        let s = s.into();
        
        if s.is_empty() {
            return Err(TopicValidationError::Empty);
        }
        
        if s.len() > 255 {
            return Err(TopicValidationError::TooLong(s.len()));
        }
        
        // Check for valid characters (alphanumeric, dots, hyphens, underscores)
        let invalid_chars: Vec<char> = s
            .chars()
            .filter(|&c| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_')
            .collect();
        
        if !invalid_chars.is_empty() {
            return Err(TopicValidationError::InvalidCharacters(
                invalid_chars.into_iter().collect()
            ));
        }
        
        Ok(Topic(s))
    }
    
    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Convert to String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Topic {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<Topic> for String {
    fn from(topic: Topic) -> Self {
        topic.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_topics() {
        assert!(Topic::new("valid_topic").is_ok());
        assert!(Topic::new("valid.topic").is_ok());
        assert!(Topic::new("valid-topic").is_ok());
        assert!(Topic::new("valid_topic.123").is_ok());
        assert!(Topic::new("ValidTopic").is_ok());
    }

    #[test]
    fn test_invalid_topics() {
        assert!(matches!(Topic::new(""), Err(TopicValidationError::Empty)));
        assert!(matches!(
            Topic::new("invalid topic"),
            Err(TopicValidationError::InvalidCharacters(_))
        ));
        assert!(matches!(
            Topic::new("invalid/topic"),
            Err(TopicValidationError::InvalidCharacters(_))
        ));
        assert!(matches!(
            Topic::new("a".repeat(256)),
            Err(TopicValidationError::TooLong(_))
        ));
    }

    #[test]
    fn test_serialization() {
        let topic = Topic::new("test_topic").unwrap();
        let serialized = serde_json::to_string(&topic).unwrap();
        assert_eq!(serialized, "\"test_topic\"");
        
        let deserialized: Topic = serde_json::from_str(&serialized).unwrap();
        assert_eq!(topic, deserialized);
    }
} 