use crate::gateway::gateway::{GateWay, GatewayError};
use crate::model::event::Event;
use prometheus::{
    register_counter_vec, register_histogram_vec, CounterVec, Encoder, HistogramVec, Opts,
    TextEncoder,
};
use std::error::Error;

pub struct MeteredEventGateway<T: GateWay> {
    gateway: T,
    counters: CounterVec,
    histogram: HistogramVec,
}

impl<T> MeteredEventGateway<T>
where
    T: GateWay,
{
    pub fn new(gateway: T) -> Result<Self, Box<dyn std::error::Error>> {
        let counter_opts = Opts::new("events_total", "Total number of events handled");

        let counters = register_counter_vec!(
            counter_opts,
            &["event_type", "event_version", "source", "result"]
        )?;

        let histogram = register_histogram_vec!(
            "event_handling_duration_seconds",
            "Histogram of event handling durations",
            &["step"]
        )?;

        Ok(MeteredEventGateway {
            gateway,
            counters,
            histogram,
        })
    }
}

impl<T> GateWay for MeteredEventGateway<T>
where
    T: GateWay,
{
    async fn handle(&self, event: &Event) -> Result<(), GatewayError> {
        let timer = self.histogram.with_label_values(&["handle"]).start_timer();
        let result = self.gateway.handle(event).await;
        timer.observe_duration();
        let event_type_label = event.event_type.as_str(); // Change as per actual field type and structure
        let event_version_label = event.event_version.as_deref().unwrap_or("unknown");
        let source_label = event.origin.as_deref().unwrap_or("unknown");
        match &result {
            Ok(_) => {
                let labels = [
                    event_type_label,
                    event_version_label,
                    source_label,
                    "success",
                ];
                self.counters.with_label_values(&labels).inc();
            }
            Err(_) => {
                let labels = [
                    event_type_label,
                    event_version_label,
                    source_label,
                    "failure",
                ];
                self.counters.with_label_values(&labels).inc();
            }
        }
        result
    }

    async fn add_topic_validation(
        &self,
        v: &crate::model::routing::TopicValidationConfig,
    ) -> Result<(), GatewayError> {
        self.gateway.add_topic_validation(v).await
    }

    async fn delete_topic_validation(&self, id: &uuid::Uuid) -> Result<(), GatewayError> {
        self.gateway.delete_topic_validation(id).await
    }

    async fn add_routing_rule(
        &self,
        rule: &crate::model::routing::TopicRoutingRule,
    ) -> Result<(), GatewayError> {
        self.gateway.add_routing_rule(rule).await
    }

    async fn get_routing_rules(
        &self,
    ) -> Result<Vec<crate::model::routing::TopicRoutingRule>, GatewayError> {
        self.gateway.get_routing_rules().await
    }

    async fn delete_routing_rule(&self, id: &uuid::Uuid) -> Result<(), GatewayError> {
        self.gateway.delete_routing_rule(id).await
    }
}
