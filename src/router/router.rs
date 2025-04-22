use crate::model::{event::Event, routing::TopicRoutingRule};

pub struct TopicRoutings {
    pub rules: Vec<TopicRoutingRule>,
}

pub trait TopicRouter {
    fn route(&self, event: &Event) -> Option<&TopicRoutingRule>;
}

impl TopicRouter for TopicRoutings {
    fn route(&self, event: &Event) -> Option<&TopicRoutingRule> {
        for rule in &self.rules {
            let type_match = rule.event_type_condition.matches(&event.event_type);
            let version_match = match (&rule.event_version_condition, &event.event_version) {
                (Some(c), Some(v)) => c.matches(&v),
                (None, _) => true,
                _ => false,
            };
            if type_match && version_match {
                return Some(&rule);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::model::{
        event::Data, event::Event, expressions::Condition, expressions::StringExpression,
    };

    use super::*;

    #[test]
    fn test_topic_router() {
        let routings = TopicRoutings {
            rules: vec![
                TopicRoutingRule {
                    id: Uuid::new_v4(),
                    order: 0,
                    topic: "topic_one".to_string(),
                    description: None,
                    event_version_condition: None,
                    event_type_condition: Condition::ONE(StringExpression::Equals {
                        value: "event_one".to_string(),
                    }),
                },
                TopicRoutingRule {
                    id: Uuid::new_v4(),
                    order: 0,
                    topic: "topic_two".to_string(),
                    description: None,
                    event_version_condition: None,
                    event_type_condition: Condition::ONE(StringExpression::Equals {
                        value: "event_two".to_string(),
                    }),
                },
            ],
        };
        let event = Event {
            id: Uuid::new_v4(),
            event_type: "event_one".to_string(),
            event_version: None,
            data: Data::String("".to_string()),
            data_type: None,
            transport_metadata: None,
            metadata: Default::default(),
            origin: None,
            timestamp: None,
        };

        let event_two = Event {
            event_type: "event_two".to_string(),
            ..event.clone()
        };

        let event_three = Event {
            event_type: "event_three".to_string(),
            ..event.clone()
        };

        assert_eq!(routings.route(&event).map(|r| r.topic), Some("topic_one"));
        assert_eq!(routings.route(&event_two).map(|r| r.topic), Some("topic_two"));
        assert_eq!(routings.route(&event_three), None);
    }

    #[test]
    fn test_topic_router_with_version_match() {
        let routings = TopicRoutings {
            rules: vec![TopicRoutingRule {
                id: Uuid::new_v4(),
                order: 0,
                topic: "topic".to_string(),
                description: None,
                event_version_condition: Some(Condition::ONE(StringExpression::Equals {
                    value: "1.0".to_string(),
                })),
                event_type_condition: Condition::ONE(StringExpression::Equals {
                    value: "event".to_string(),
                }),
            }],
        };
        let event = Event {
            id: Uuid::new_v4(),
            event_type: "event".to_string(),
            event_version: None,
            data: Data::String("".to_string()),
            data_type: None,
            metadata: Default::default(),
            transport_metadata: None,
            origin: None,
            timestamp: None,
        };

        let event_two = Event {
            event_version: Some("1.0".to_string()),
            ..event.clone()
        };

        let event_three = Event {
            event_type: "event_three".to_string(),
            event_version: Some("3.0".to_string()),
            ..event.clone()
        };

        assert_eq!(routings.route(&event), None);
        assert_eq!(routings.route(&event_two).map(|r| r.topic), Some("topic"));
        assert_eq!(routings.route(&event_three), None);
    }
}
