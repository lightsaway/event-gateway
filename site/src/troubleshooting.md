# Troubleshooting

## HTTP 406 for every event

No routing rule matched. List rules:

```bash
curl http://localhost:8080/api/v1/routing-rules
```

Check event type, optional version, rule order, and condition serialization.

## PGMQ reports queue errors

Queues are not created automatically:

```sql
SELECT pgmq.create('outbox');
```

The routing topic must exactly match the queue name.

## Environment override is ignored

Use double underscores between sections:

```bash
APP_GATEWAY__PUBLISHER__CONNECTION_URL=...
```

Do not use `APP_GATEWAY_PUBLISHER_CONNECTION_URL`.

## Health works but publishing fails

`/health-check` is liveness-only. Inspect logs and test the configured
publisher directly.

## Docker build fails on native dependencies

The Dockerfile installs static OpenSSL, zlib, curl, CMake, and compiler
packages required by `librdkafka`. Build from the repository root so UI and
migration files are in the build context.

## Probe returns 401

JWT authorization currently covers health and metrics. Supply a token or
disable JWT for the probe-facing deployment until operational routes are
separated.
