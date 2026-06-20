# Delivery Semantics

Event Gateway is synchronous up to the selected publisher's acknowledgment
boundary.

## PGMQ

Success means PostgreSQL committed `pgmq.send`. The event is durable in the
queue, but `pgmq-relay` and the destination broker still operate
asynchronously. If the HTTP response is lost after commit, a producer retry can
create a duplicate.

## Kafka

Success means librdkafka completed its delivery future according to configured
acks and timeouts. This does not mean a consumer processed the record.

## MQTT

Success means the async client accepted the publish request. The current
implementation does not await the complete QoS acknowledgment handshake.

## Producer guidance

- assign a stable event UUID before retrying;
- treat retries after timeout or connection loss as potentially duplicating;
- make consumers idempotent using `event.id`;
- use PGMQ when a durable handoff before broker delivery is required;
- do not use event order as a global guarantee.

The service does not provide exactly-once end-to-end processing.
