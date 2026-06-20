# MQTT Publisher

```toml
[gateway.publisher]
type = "mqtt"
host = "mosquitto"
port = 1883
client_id = "event-gateway"
keep_alive = 30
clean_session = false
qos = "atleastonce"
retain = false
```

QoS values deserialize in lowercase:

- `atmostonce`
- `atleastonce`
- `exactlyonce`

The MQTT event loop runs as a background task. `publish_one` confirms that the
request entered the async client; it does not wait for the complete MQTT QoS
handshake. Event-loop failures are currently logged and do not change process
health.

For a durable ingress boundary with explicit database acknowledgment, use
PGMQ instead.
