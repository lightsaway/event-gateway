# Schema Validation

Validations are attached to a topic and selected by event type and optional
event version.

Create a validation:

```bash
curl -X POST http://localhost:8080/api/v1/topic-validations \
  -H 'content-type: application/json' \
  -d '{
    "topic": "orders",
    "schema": {
      "name": "order-created-v1",
      "description": "Required order fields",
      "eventType": "order.created",
      "eventVersion": "1",
      "metadata": {},
      "schema": {
        "type": "json",
        "data": {
          "type": "object",
          "properties": {
            "order_id": {"type": "integer"}
          },
          "required": ["order_id"]
        }
      }
    }
  }'
```

The gateway compiles JSON schemas when configuration is loaded. An event must
pass every matching schema for its selected topic.

## Important behavior

- only `data.type = "json"` is validated;
- string and binary event data bypass JSON Schema;
- no matching schema means no validation;
- validation failure returns HTTP 400;
- schema error details are logged but not returned to the caller.
