# Architecture

```text
HTTP router
  |
  +-- request metadata enrichment
  |
  v
EventGateway
  |
  +-- load ordered routing rules
  +-- select first matching rule
  +-- load schemas for selected topic
  +-- validate JSON payload
  |
  v
Publisher
  +-- Kafka
  +-- MQTT
  +-- PGMQ
  +-- no-op
```

## Configuration repository

The `Storage` interface contains routing rules and topic validation
configuration. Available implementations are:

- in-memory, optionally initialized from JSON;
- files under a configured directory;
- PostgreSQL with an in-process read cache.

Events are never written to these stores.

## Routing

Rules are sorted by `order`; the first matching rule wins. Ties are stable only
where the backend provides a deterministic secondary order. Avoid duplicate
order values when precedence matters.

## Validation

Validation is performed after routing because schemas are attached to the
destination topic. Only JSON event data is schema-validated. String and binary
payloads bypass JSON Schema validation.

## UI

The React UI is compiled during the build and embedded in the Rust binary.
Unknown non-file paths fall back to `index.html` for client-side routing.

## Current operational limits

- health is liveness-only;
- cached PostgreSQL refresh tasks are internal background tasks;
- file storage performs blocking filesystem operations;
- authentication is applied to the full API router, including health and
  metrics, when JWT is enabled.
