# First Event

This walkthrough uses in-memory configuration and the no-op publisher. The
publisher logs the serialized event instead of sending it to a broker.

Start the gateway:

```bash
APP_CONFIG_PATH=./configs/minimal-config.toml cargo run --locked
```

Create a routing rule:

```bash
curl -X POST http://localhost:8080/api/v1/routing-rules \
  -H 'content-type: application/json' \
  -d '{
    "order": 0,
    "topic": "orders",
    "description": "Route order events",
    "groupMetadataField": "order_id",
    "eventTypeCondition": {
      "type": "equals",
      "value": "order.created"
    }
  }'
```

Submit an event:

```bash
curl -i -X POST http://localhost:8080/api/v1/event \
  -H 'content-type: application/json' \
  -d '{
    "id": "11111111-1111-4111-8111-111111111111",
    "eventType": "order.created",
    "eventVersion": "1",
    "metadata": {
      "tenant_id": "acme",
      "order_id": "order-123"
    },
    "data": {
      "type": "json",
      "content": {
        "order_id": 123
      }
    },
    "origin": "checkout"
  }'
```

Expected response:

```http
HTTP/1.1 200 OK
content-type: application/json

{"status": "success"}
```

HTTP 200 means the configured publisher accepted the event according to its
documented acknowledgment boundary. It does not mean a downstream consumer
processed it.
