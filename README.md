# Event Gateway

[![CI](https://github.com/lightsaway/event-gateway/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/lightsaway/event-gateway/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/lightsaway/event-gateway?display_name=tag&sort=semver)](https://github.com/lightsaway/event-gateway/releases/latest)

Event Gateway accepts events over HTTP, validates JSON payloads, applies
ordered routing rules, and publishes to PGMQ, Kafka, or MQTT.

```text
HTTP -> Event Gateway -> PGMQ -> pgmq-relay -> Kafka / RabbitMQ / NATS
```

Full documentation: <https://lightsaway.github.io/event-gateway/>

## Architecture

```mermaid
flowchart LR
    Client[HTTP client]
    API[Axum API]
    Auth[Optional JWT validation]
    Gateway[Event Gateway]
    Config[(Routing rules<br/>and schemas)]
    Router[Route and validate]
    Publisher{Publisher}
    PGMQ[(PostgreSQL<br/>PGMQ)]
    Kafka[Kafka]
    MQTT[MQTT]
    Relay[pgmq-relay]
    Brokers[Kafka / RabbitMQ / NATS]

    Client -->|POST /event| API
    API --> Auth
    Auth --> Gateway
    Config --> Gateway
    Gateway --> Router
    Router --> Publisher
    Publisher --> PGMQ
    Publisher --> Kafka
    Publisher --> MQTT
    PGMQ --> Relay
    Relay --> Brokers
```

JWT validation applies only to event ingestion. Health, metrics, and
configuration-management endpoints should be protected at the network or
reverse-proxy layer.

## Quick Start

```bash
docker build -t event-gateway:local .
docker run --rm -p 8080:8080 event-gateway:local
curl http://localhost:8080/api/v1/health-check
```

The bundled configuration uses the no-op publisher and has no routing rules.
Follow the [first-event walkthrough](https://lightsaway.github.io/event-gateway/first-event.html)
to create a rule and submit an event.

## PGMQ Pipeline

Configure the gateway:

```toml
[gateway.publisher]
type = "pgmq"
connection_url = "postgres://event_gateway:secret@postgres:5432/app"
max_connections = 10
delay_seconds = 0
group_metadata_field = "aggregate_id"
```

The routing topic becomes the PGMQ queue name. See the
[PGMQ publisher guide](https://lightsaway.github.io/event-gateway/publishers/pgmq.html)
for queue provisioning, per-routing-rule group overrides, and relay configuration.

## Build

```bash
make frontend-build
cargo build --release --locked
```

The project uses Rust 1.96 and Node.js 22.

## Container Images

Tagged releases publish multi-architecture images to GitHub Container
Registry:

```bash
docker pull ghcr.io/lightsaway/event-gateway:latest
docker pull ghcr.io/lightsaway/event-gateway:0.1.0
```

## Checks

```bash
make ci-check
```

See [Contributing](https://lightsaway.github.io/event-gateway/contributing.html)
and [Release Process](https://lightsaway.github.io/event-gateway/release.html).
