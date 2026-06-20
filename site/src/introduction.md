# Event Gateway

Event Gateway accepts events over HTTP, selects a destination using ordered
routing rules, optionally validates the event payload, and publishes it to
Kafka, MQTT, or PGMQ.

The common durable pipeline is:

```text
HTTP producer
    |
    v
Event Gateway
    |
    v
PGMQ queue in PostgreSQL
    |
    v
pgmq-relay
    |
    +--> Kafka
    +--> RabbitMQ
    +--> NATS
```

Event Gateway stores routing rules and schema configuration. It does **not**
store accepted events. Event durability is determined by the selected
publisher.

## Use it when

- producers need one stable HTTP contract;
- routing must be changed without redeploying producers;
- payloads need JSON Schema validation before entering a broker;
- PGMQ should provide a durable boundary before downstream delivery.

## Do not assume

- HTTP 200 means a downstream consumer processed the event;
- `/health-check` verifies PostgreSQL or broker readiness;
- the gateway provides exactly-once delivery;
- routing-rule updates are transactional with event publication.

Read [Delivery Semantics](delivery-semantics.md) before using the service for
critical traffic.
