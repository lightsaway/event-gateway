use crate::configuration::ApiConfig;
use crate::gateway::gateway::GateWay;
use crate::model::expressions::Condition;
use crate::model::routing::{DataSchema, TopicRoutingRule, TopicValidationConfig};
use crate::{gateway::gateway::EventGateway, model::event::Event};
use axum::extract::Path;
use axum::routing::delete;
use axum::{
    body::Body,
    extract::{Extension, State},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTopicValidationRequest {
    topic: String,
    schema: DataSchema,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateRoutingRuleRequest {
    order: i32,
    topic: String,
    event_type_condition: Condition,
    event_version_condition: Option<Condition>,
    description: Option<String>,
}

pub fn app_router(
    service: Arc<crate::gateway::gateway::EventGateway>,
    config: &ApiConfig,
) -> Router {
    let routes = Router::new()
        .route("/event", post(handle_event))
        .route("/routing-rules", get(read_rules))
        .route("/routing-rules", post(create_routing_rule))
        .route("/routing-rules/:id", delete(create_routing_rule))
        .route("/topic-validations", get(read_topic_validations))
        .route("/topic-validations", post(create_topic_validation))
        .route("/topic-validations/:id", delete(delete_topic_validation))
        .route("/health-check", get(health_check))
        .with_state(service);
    Router::new().nest(config.prefix.as_str(), routes)
}

async fn health_check() -> &'static str {
    r#"{ "status" : "healthy"}"#
}

async fn handle_event(
    State(service): State<Arc<EventGateway>>,
    Json(event): Json<Event>,
) -> Result<Response, Response> {
    let result = service.handle(&event).await;
    match result {
        Ok(_) => Ok(Response::builder().status(200).body(Body::empty()).unwrap()),
        Err(err) => match err {
            crate::gateway::gateway::GatewayError::SchemaInvalid(err) => Ok(Response::builder()
                .status(400)
                .body(Body::from(r#"{"error": "schema validation failed"}"#))
                .unwrap()),
            crate::gateway::gateway::GatewayError::NoTopicToRoute(err) => Ok(Response::builder()
                .status(406)
                .body(Body::from(r#"{"error": "no destination found"}"#))
                .unwrap()),
            crate::gateway::gateway::GatewayError::InternalError(err) => {
                Ok(Response::builder().status(500).body(Body::empty()).unwrap())
            }
        },
    }
}

async fn create_routing_rule(
    State(service): State<Arc<EventGateway>>,
    Json(request): Json<CreateRoutingRuleRequest>,
) -> Result<Response, Response> {
    let rule = TopicRoutingRule {
        id: Uuid::new_v4(),
        order: request.order,
        topic: request.topic,
        event_type_condition: request.event_type_condition,
        event_version_condition: request.event_version_condition,
        description: request.description,
    };
    let result = service.add_routing_rule(&rule).await;
    match result {
        Ok(_) => Ok(Response::builder().status(204).body(Body::empty()).unwrap()),
        Err(err) => Ok(Response::builder().status(500).body(Body::empty()).unwrap()),
    }
}

async fn delete_routing_rule(
    State(service): State<Arc<EventGateway>>,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    let result = service.delete_routing_rule(&id).await;
    match result {
        Ok(_) => Ok(Response::builder().status(204).body(Body::empty()).unwrap()),
        Err(err) => Ok(Response::builder().status(500).body(Body::empty()).unwrap()),
    }
}

async fn create_topic_validation(
    State(service): State<Arc<EventGateway>>,
    Json(request): Json<CreateTopicValidationRequest>,
) -> Result<Response, Response> {
    let validation = TopicValidationConfig {
        id: Uuid::new_v4(),
        topic: request.topic,
        schema: request.schema,
    };
    let result = service.add_topic_validation(&validation).await;
    match result {
        Ok(_) => Ok(Response::builder().status(204).body(Body::empty()).unwrap()),
        Err(err) => Ok(Response::builder().status(500).body(Body::empty()).unwrap()),
    }
}

async fn read_topic_validations(
    State(service): State<Arc<EventGateway>>,
) -> Result<Response, Response> {
    let result = service.get_topic_validations().await;
    match result {
        Ok(validations) => Ok(Response::builder()
            .status(200)
            .body(Body::from(serde_json::to_string(&validations).unwrap()))
            .unwrap()),
        Err(err) => Ok(Response::builder().status(500).body(Body::empty()).unwrap()),
    }
}

async fn delete_topic_validation(
    State(service): State<Arc<EventGateway>>,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    let result = service.delete_topic_validation(&id).await;
    match result {
        Ok(_) => Ok(Response::builder().status(204).body(Body::empty()).unwrap()),
        Err(err) => Ok(Response::builder().status(500).body(Body::empty()).unwrap()),
    }
}

async fn read_rules(State(service): State<Arc<EventGateway>>) -> Result<Response, Response> {
    let result = service.get_routing_rules().await;
    match result {
        Ok(rules) => Ok(Response::builder()
            .status(200)
            .body(Body::from(serde_json::to_string(&rules).unwrap()))
            .unwrap()),
        Err(err) => Ok(Response::builder().status(500).body(Body::empty()).unwrap()),
    }
}
