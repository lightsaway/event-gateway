# Event Gateway

Event Gateway accepts events over HTTP, validates JSON payloads, applies
ordered routing rules, and publishes to PGMQ, Kafka, or MQTT.

```text
HTTP -> Event Gateway -> PGMQ -> pgmq-relay -> Kafka / RabbitMQ / NATS
```

Full documentation: <https://lightsaway.github.io/event-gateway/>

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
```

The routing topic becomes the PGMQ queue name. See the
[PGMQ publisher guide](https://lightsaway.github.io/event-gateway/publishers/pgmq.html)
for queue provisioning and relay configuration.

## Build

```bash
make frontend-build
cargo build --release --locked
```

The project uses Rust 1.88 and Node.js 22.

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
