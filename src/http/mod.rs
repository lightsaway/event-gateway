use crate::configuration::ApiConfig;
use crate::gateway::gateway::GateWay;
use crate::model::event::Event;
use crate::model::expressions::Condition;
use crate::model::routing::{DataSchema, TopicRoutingRule, TopicValidationConfig};
use crate::model::topic::Topic;
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
use hyper::header::USER_AGENT;
use hyper::StatusCode;
use jwt_authorizer::{
    layer::AuthorizationLayer, Authorizer, IntoLayer, JwtAuthorizer, RegisteredClaims,
};
use log::{error, warn};
use prometheus::{Encoder, TextEncoder};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

type GatewayService = dyn GateWay + Send + Sync;

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
    service: Arc<GatewayService>,
    config: &ApiConfig,
    metrics_enabled: bool,
) -> Result<Router, Box<dyn std::error::Error>> {
    let authorization = match &config.jwt_auth {
        Some(cfg) => {
            let authorizer: Authorizer<RegisteredClaims> =
                JwtAuthorizer::from_jwks_url(&cfg.jwks_url).build().await?;
            Some(authorizer.into_layer())
        }
        None => None,
    };

    Ok(build_router(
        service,
        config.prefix.as_deref().unwrap_or("/"),
        metrics_enabled,
        authorization,
    ))
}

fn build_router(
    service: Arc<GatewayService>,
    prefix: &str,
    metrics_enabled: bool,
    authorization: Option<AuthorizationLayer<RegisteredClaims>>,
) -> Router {
    let mut public_routes = Router::new()
        .route("/routing-rules", get(read_rules))
        .route("/routing-rules", post(create_routing_rule))
        .route("/routing-rules/:id", put(update_routing_rule))
        .route("/routing-rules/:id", delete(delete_routing_rule))
        .route("/topic-validations", get(read_topic_validations))
        .route("/topic-validations", post(create_topic_validation))
        .route("/topic-validations/:id", delete(delete_topic_validation))
        .route("/health-check", get(health_check));

    if metrics_enabled {
        public_routes = public_routes.route("/metrics", get(metrics_endpoint));
    }

    let ingestion_routes = match authorization {
        Some(layer) => Router::new()
            .route("/event", post(handle_event))
            .with_state(Arc::clone(&service))
            .layer(layer),
        None => Router::new()
            .route("/event", post(handle_event))
            .with_state(Arc::clone(&service))
            .layer(Extension(Option::<RegisteredClaims>::None)),
    };

    let routes = public_routes.with_state(service).merge(ingestion_routes);

    Router::new()
        .nest(prefix, routes)
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

async fn metrics_endpoint() -> Response {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(Body::from(buffer))
        .unwrap()
}

async fn handle_event(
    State(service): State<Arc<GatewayService>>,
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
            crate::gateway::gateway::GatewayError::SchemaInvalid(err) => {
                warn!("Event rejected by schema validation: {err}");
                Ok(Response::builder()
                    .status(400)
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"error": "schema validation failed"}"#))
                    .unwrap())
            }
            crate::gateway::gateway::GatewayError::NoTopicToRoute(err) => {
                warn!("Event has no routing destination: {err}");
                Ok(Response::builder()
                    .status(406)
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"error": "no destination found"}"#))
                    .unwrap())
            }
            crate::gateway::gateway::GatewayError::InternalError(err) => {
                error!("Failed to handle event: {err}");
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
    State(service): State<Arc<GatewayService>>,
    Json(request): Json<CreateRoutingRuleRequest>,
) -> Result<Response, Response> {
    let topic = Topic::new(&request.topic).map_err(|e| {
        Response::builder()
            .status(400)
            .header("Content-Type", "application/json")
            .body(Body::from(format!(r#"{{"error": "Invalid topic: {e}"}}"#)))
            .unwrap()
    })?;

    let rule = TopicRoutingRule {
        id: Uuid::new_v4(),
        order: request.order,
        topic,
        event_type_condition: request.event_type_condition,
        event_version_condition: request.event_version_condition,
        description: request.description,
    };
    let result = service.add_routing_rule(&rule).await;
    match result {
        Ok(_) => Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "success"}"#))
            .unwrap()),
        Err(err) => {
            error!("Failed to create routing rule: {err}");
            Ok(Response::builder()
                .status(500)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error": "internal server error"}"#))
                .unwrap())
        }
    }
}

async fn update_routing_rule(
    State(service): State<Arc<GatewayService>>,
    Path(id): Path<Uuid>,
    Json(request): Json<CreateRoutingRuleRequest>,
) -> Result<Response, Response> {
    let topic = Topic::new(&request.topic).map_err(|e| {
        Response::builder()
            .status(400)
            .header("Content-Type", "application/json")
            .body(Body::from(format!(r#"{{"error": "Invalid topic: {e}"}}"#)))
            .unwrap()
    })?;

    let rule = TopicRoutingRule {
        id,
        order: request.order,
        topic,
        event_type_condition: request.event_type_condition,
        event_version_condition: request.event_version_condition,
        description: request.description,
    };
    let result = service.update_routing_rule(id, &rule).await;
    match result {
        Ok(_) => Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "success"}"#))
            .unwrap()),
        Err(err) => {
            error!("Failed to update routing rule {id}: {err}");
            Ok(Response::builder()
                .status(500)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error": "internal server error"}"#))
                .unwrap())
        }
    }
}

async fn delete_routing_rule(
    State(service): State<Arc<GatewayService>>,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    let result = service.delete_routing_rule(&id).await;
    match result {
        Ok(_) => Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "success"}"#))
            .unwrap()),
        Err(err) => {
            error!("Failed to delete routing rule {id}: {err}");
            Ok(Response::builder()
                .status(500)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error": "internal server error"}"#))
                .unwrap())
        }
    }
}

async fn create_topic_validation(
    State(service): State<Arc<GatewayService>>,
    Json(request): Json<CreateTopicValidationRequest>,
) -> Result<Response, Response> {
    let topic = Topic::new(&request.topic).map_err(|e| {
        Response::builder()
            .status(400)
            .header("Content-Type", "application/json")
            .body(Body::from(format!(r#"{{"error": "Invalid topic: {e}"}}"#)))
            .unwrap()
    })?;

    let validation = TopicValidationConfig {
        id: Uuid::new_v4(),
        topic,
        schema: request.schema,
    };
    let result = service.add_topic_validation(&validation).await;
    match result {
        Ok(_) => Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "success"}"#))
            .unwrap()),
        Err(err) => {
            error!("Failed to create topic validation: {err}");
            Ok(Response::builder()
                .status(500)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error": "internal server error"}"#))
                .unwrap())
        }
    }
}

async fn read_topic_validations(
    State(service): State<Arc<GatewayService>>,
) -> Result<Response, Response> {
    let result = service.get_topic_validations().await;
    match result {
        Ok(validations) => Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&validations).unwrap()))
            .unwrap()),
        Err(err) => {
            error!("Failed to read topic validations: {err}");
            Ok(Response::builder().status(500).body(Body::empty()).unwrap())
        }
    }
}

async fn delete_topic_validation(
    State(service): State<Arc<GatewayService>>,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    let result = service.delete_topic_validation(&id).await;
    match result {
        Ok(_) => Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status": "success"}"#))
            .unwrap()),
        Err(err) => {
            error!("Failed to delete topic validation {id}: {err}");
            Ok(Response::builder()
                .status(500)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error": "internal server error"}"#))
                .unwrap())
        }
    }
}

async fn read_rules(State(service): State<Arc<GatewayService>>) -> Result<Response, Response> {
    let result = service.get_routing_rules().await;
    match result {
        Ok(rules) => Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&rules).unwrap()))
            .unwrap()),
        Err(err) => {
            error!("Failed to read routing rules: {err}");
            Ok(Response::builder().status(500).body(Body::empty()).unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::gateway::EventGateway;
    use crate::publisher::publisher::NoOpPublisher;
    use crate::store::storage::InMemoryStorage;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn jwt_protects_only_event_ingestion() {
        let service: Arc<GatewayService> = Arc::new(EventGateway::new(
            Box::new(NoOpPublisher),
            Box::new(InMemoryStorage::new()),
        ));
        let authorizer: Authorizer<RegisteredClaims> = JwtAuthorizer::from_secret("test-secret")
            .build()
            .await
            .unwrap();
        let app = build_router(service, "/api/v1", true, Some(authorizer.into_layer()));

        for path in [
            "/api/v1/health-check",
            "/api/v1/metrics",
            "/api/v1/routing-rules",
            "/api/v1/topic-validations",
        ] {
            let response = app
                .clone()
                .oneshot(Request::get(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK, "{path}");
        }

        let response = app
            .oneshot(
                Request::post("/api/v1/event")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
