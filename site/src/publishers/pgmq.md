# PGMQ Publisher

PGMQ is the recommended durable boundary for the gateway-to-relay pipeline.

```toml
[gateway.publisher]
type = "pgmq"
connection_url = "postgres://event_gateway:secret@postgres:5432/app"
max_connections = 10
delay_seconds = 0
```

The routing rule's `topic` is used as the PGMQ queue name. Provision queues
before starting traffic:

```sql
CREATE EXTENSION IF NOT EXISTS pgmq;
SELECT pgmq.create('outbox');
```

The publisher writes:

- the complete event as the PGMQ JSON message;
- `event_id`, `event_type`, and `event_version` as PGMQ headers.

Configure `pgmq-relay` against the same database and queue:

```toml
[pgmq]
connection_url = "postgres://relay:secret@postgres:5432/app"

[[queues]]
queue_name = "outbox"
destination_topic = "events.orders"
```

HTTP 200 is returned only after PostgreSQL completes `pgmq.send`. A missing
queue, database outage, or SQL error returns HTTP 500.
