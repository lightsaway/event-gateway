# Routing Rules

Rules are evaluated in ascending `order`. The first match determines the
publisher topic or PGMQ queue.

```json
{
  "order": 0,
  "topic": "orders",
  "description": "Version 1 order events",
  "groupMetadataField": "order_id",
  "eventTypeCondition": {
    "type": "startsWith",
    "value": "order."
  },
  "eventVersionCondition": {
    "type": "equals",
    "value": "1"
  }
}
```

`eventVersionCondition` is optional. If present, an event without
`eventVersion` does not match.

`groupMetadataField` is optional. For the PGMQ publisher it overrides the
global `group_metadata_field` and copies that event metadata value into the
`x-pgmq-group` header.

## Conditions

String expressions:

- `equals`
- `startsWith`
- `endsWith`
- `contains`
- `regexMatch`

Condition composition:

```json
{
  "and": [
    {"type": "startsWith", "value": "order."},
    {"not": {"type": "endsWith", "value": ".test"}}
  ]
}
```

The wire representation follows the existing serde model. Test complex
conditions against a non-production instance before applying them.

## Failure behavior

- no matching rule: HTTP 406;
- malformed topic: HTTP 400 when creating or updating a rule;
- storage failure: HTTP 500;
- publisher failure: HTTP 500.
