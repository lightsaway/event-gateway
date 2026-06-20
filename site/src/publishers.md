# Publishers

Exactly one publisher is configured for a gateway process.

| Type | Destination name | Success boundary |
|---|---|---|
| `noOp` | routing topic | event serialized and logged |
| `pgmq` | PGMQ queue | `pgmq.send` committed in PostgreSQL |
| `kafka` | Kafka topic | librdkafka delivery future completed |
| `mqtt` | MQTT topic | publish request accepted by the async client |

Publisher success is not consumer acknowledgment. See
[Delivery Semantics](delivery-semantics.md).
