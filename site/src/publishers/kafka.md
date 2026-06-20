# Kafka Publisher

```toml
[gateway.publisher]
type = "kafka"
brokers = ["kafka-1:9092", "kafka-2:9092"]
compression = "gzip"
client_id = "event-gateway"
required_acks = "all"
conn_idle_timeout = "5m"
message_timeout = "5s"
ack_timeout = "5s"
metadata_field_as_key = "tenant_id"
```

Compression values:

- `none`
- `gzip`
- `snappy`

Acknowledgment values:

- `none`
- `one`
- `all`

The producer waits for librdkafka's delivery result. `message_timeout` bounds
overall delivery; `ack_timeout` maps to Kafka request timeout.

The message key is selected from `metadata_field_as_key`. If the field is
missing, the event UUID is used.

This publisher does not implement application-level retries or a dead-letter
queue. Kafka and librdkafka configuration determine broker retries.
