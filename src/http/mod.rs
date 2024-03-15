use crate::configuration::ApiConfig;
use crate::gateway::gateway::GateWay;
use crate::{gateway::gateway::EventGateway, model::event::Event};
use axum::{
    body::Body,
    extract::{Extension, State},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;

async fn health_check() -> &'static str {
    r#"{ "status" : "healthy"}"#
}

pub fn app_router(
    service: Arc<crate::gateway::gateway::EventGateway>,
    config: &ApiConfig,
) -> Router {
    let routes = Router::new()
        .route("/health-check", get(health_check))
        .route("/event", post(handle_event))
        .route("/routing-rules", get(read_rules))
        .with_state(service);
    Router::new().nest(config.prefix.as_str(), routes)
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
