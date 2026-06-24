# PGMQ Publisher

PGMQ is the recommended durable boundary for the gateway-to-relay pipeline.

```toml
[gateway.publisher]
type = "pgmq"
connection_url = "postgres://event_gateway:secret@postgres:5432/app"
max_connections = 10
delay_seconds = 0
group_metadata_field = "aggregate_id"
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

## FIFO group headers

`group_metadata_field` copies the selected event metadata value into the
`x-pgmq-group` PGMQ header:

```json
{
  "eventType": "order.created",
  "metadata": {
    "aggregate_id": "order-42"
  }
}
```

Each routing rule can override the default:

```json
{
  "topic": "outbox",
  "groupMetadataField": "order_id",
  "eventTypeCondition": {
    "type": "equals",
    "value": "order.created"
  }
}
```

The matched rule's `groupMetadataField` takes precedence over the publisher
default. This supports pattern-based rules and rules that route similar events
to queues with different grouping requirements.

When a grouping field is configured, that metadata entry is required and must
be non-empty. The gateway rejects the event instead of publishing it without a
group header.

Omit the rule override and publisher default when the queue does not require
grouped FIFO processing.

Configure `pgmq-relay` against the same database and queue:

```toml
[pgmq]
connection_url = "postgres://relay:secret@postgres:5432/app"

[[queues]]
queue_name = "outbox"
destination_topic = "events.orders"
fetch_mode = "read_grouped_head_with_poll"
key_field = "x-pgmq-group"
```

Using the same group as the relay message key keeps each group on one Kafka
partition. Horizontal relay scaling can process different groups concurrently,
while at-least-once redelivery still requires consumers to handle duplicates
and stale sequence numbers.

HTTP 200 is returned only after PostgreSQL completes `pgmq.send`. A missing
queue, database outage, or SQL error returns HTTP 500.
