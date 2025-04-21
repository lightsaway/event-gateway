use crate::configuration::ApiConfig;
use crate::gateway::gateway::GateWay;
use crate::model::expressions::Condition;
use crate::model::routing::{DataSchema, TopicRoutingRule, TopicValidationConfig};
use crate::{gateway::gateway::EventGateway, model::event::Event};
use axum::async_trait;
use axum::extract::{FromRequestParts, Path, Request};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::routing::delete;
use axum::{
    body::Body,
    extract::{Extension, State},
    response::Response,
    routing::{get, post, put},
    Json, Router,
};
use futures::Future;
use hyper::header::USER_AGENT;
use hyper::StatusCode;
use jwks::Jwks;
use jwt_authorizer::{
    Authorizer, IntoLayer, JwtAuthorizer, Refresh, RefreshStrategy, RegisteredClaims,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::{Service, ServiceBuilder};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct RequestMetadata {
    originator_ip: Option<String>,
    user_agent: Option<String>,
}

impl RequestMetadata {
    fn to_hash_map_string(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        // Insert only if the value is Some
        if let Some(ref value) = self.originator_ip {
            map.insert("originatorIp".to_string(), value.clone());
        }
        if let Some(ref value) = self.user_agent {
            map.insert("userAgent".to_string(), value.clone());
        }
        map
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequestMetadata {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let originator_ip = parts
            .headers
            .get("x-forwarded-for")
            .or_else(|| parts.headers.get("x-real-ip"))
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_owned())
            .or_else(|| {
                parts
                    .extensions
                    .get::<SocketAddr>()
                    .map(|addr| addr.ip().to_string())
            });

        let user_agent = parts
            .headers
            .get(USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_owned());

        Ok(RequestMetadata {
            originator_ip,
            user_agent,
        })
    }
}

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

pub async fn app_router(
    service: Arc<crate::gateway::gateway::EventGateway>,
    config: &ApiConfig,
) -> Router {
    let routes = Router::new()
        .route("/event", post(handle_event))
        .route("/routing-rules", get(read_rules))
        .route("/routing-rules", post(create_routing_rule))
        .route("/routing-rules/:id", put(update_routing_rule))
        .route("/routing-rules/:id", delete(delete_routing_rule))
        .route("/topic-validations", get(read_topic_validations))
        .route("/topic-validations", post(create_topic_validation))
        .route("/topic-validations/:id", delete(delete_topic_validation))
        .route("/health-check", get(health_check))
        .with_state(service);
    let prefix = config.prefix.clone().unwrap_or("/".to_string());
    let router = match &config.jwt_auth {
        Some(cfg) => {
            let jwks = Jwks::from_jwks_url(&cfg.jwks_url).await.unwrap();
            let authorizer: Authorizer<RegisteredClaims> =
                JwtAuthorizer::from_jwks_url(&cfg.jwks_url)
                    .build()
                    .await
                    .unwrap();
            Router::new()
                .nest(&prefix, routes)
                .layer(authorizer.into_layer())
        }
        None => Router::new()
            .nest(&prefix, routes)
            .layer(Extension(Option::<RegisteredClaims>::None)),
    };
    router
        .layer(axum::middleware::from_fn(extract_request_metadata))
        .layer(TraceLayer::new_for_http())
}

async fn extract_request_metadata(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let originator_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_owned())
        .or_else(|| {
            req.extensions()
                .get::<SocketAddr>()
                .map(|addr| addr.ip().to_string())
        });

    let user_agent = req
        .headers()
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_owned());

    let metadata = RequestMetadata {
        originator_ip,
        user_agent,
    };
    req.extensions_mut().insert(metadata);

    Ok(next.run(req).await)
}

async fn health_check() -> Response {
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{ "status" : "healthy"}"#))
        .unwrap()
}

async fn handle_event(
    State(service): State<Arc<EventGateway>>,
    Extension(claims): Extension<Option<RegisteredClaims>>,
    Extension(metadata): Extension<RequestMetadata>,
    Json(mut event): Json<Event>,
) -> Result<Response, Response> {
    let mut transport_meta = HashMap::new();
    if let Some(claims) = claims {
        if let Some(sub) = claims.sub {
            transport_meta.insert("jwt_sub".to_string(), sub);
        }
        if let Some(iss) = claims.iss {
            transport_meta.insert("jwt_iss".to_string(), iss);
        }
    }
    transport_meta.extend(metadata.to_hash_map_string());
    event.transport_metadata = Some(transport_meta);
    let result = service.handle(&event).await;
    match result {
        Ok(_) => Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "success"}"#))
            .unwrap()),
        Err(err) => match err {
            crate::gateway::gateway::GatewayError::SchemaInvalid(err) => Ok(Response::builder()
                .status(400)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error": "schema validation failed"}"#))
                .unwrap()),
            crate::gateway::gateway::GatewayError::NoTopicToRoute(err) => Ok(Response::builder()
                .status(406)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error": "no destination found"}"#))
                .unwrap()),
            crate::gateway::gateway::GatewayError::InternalError(err) => {
                Ok(Response::builder()
                    .status(500)
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"error": "internal server error"}"#))
                    .unwrap())
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
        Ok(_) => Ok(Response::builder()
            .status(204)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "success"}"#))
            .unwrap()),
        Err(err) => Ok(Response::builder()
            .status(500)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error": "internal server error"}"#))
            .unwrap()),
    }
}

async fn update_routing_rule(
    State(service): State<Arc<EventGateway>>,
    Path(id): Path<Uuid>,
    Json(request): Json<CreateRoutingRuleRequest>,
) -> Result<Response, Response> {
    let rule = TopicRoutingRule {
        id,
        order: request.order,
        topic: request.topic,
        event_type_condition: request.event_type_condition,
        event_version_condition: request.event_version_condition,
        description: request.description,
    };
    let result = service.update_routing_rule(id, &rule).await;
    match result {
        Ok(_) => Ok(Response::builder()
            .status(204)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "success"}"#))
            .unwrap()),
        Err(err) => Ok(Response::builder()
            .status(500)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error": "internal server error"}"#))
            .unwrap()),
    }
}

async fn delete_routing_rule(
    State(service): State<Arc<EventGateway>>,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    let result = service.delete_routing_rule(&id).await;
    match result {
        Ok(_) => Ok(Response::builder()
            .status(204)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "success"}"#))
            .unwrap()),
        Err(err) => Ok(Response::builder()
            .status(500)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error": "internal server error"}"#))
            .unwrap()),
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
        Ok(_) => Ok(Response::builder()
            .status(204)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "success"}"#))
            .unwrap()),
        Err(err) => Ok(Response::builder()
            .status(500)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error": "internal server error"}"#))
            .unwrap()),
    }
}

async fn read_topic_validations(
    State(service): State<Arc<EventGateway>>,
) -> Result<Response, Response> {
    let result = service.get_topic_validations().await;
    match result {
        Ok(validations) => Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
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
        Ok(_) => Ok(Response::builder()
            .status(204)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "success"}"#))
            .unwrap()),
        Err(err) => Ok(Response::builder()
            .status(500)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error": "internal server error"}"#))
            .unwrap()),
    }
}

async fn read_rules(State(service): State<Arc<EventGateway>>) -> Result<Response, Response> {
    let result = service.get_routing_rules().await;
    match result {
        Ok(rules) => Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&rules).unwrap()))
            .unwrap()),
        Err(err) => Ok(Response::builder().status(500).body(Body::empty()).unwrap()),
    }
}
