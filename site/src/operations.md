# Operations

## Health

`GET /health-check` returns:

```json
{"status": "healthy"}
```

This is a liveness response. It does not check:

- configuration storage;
- PGMQ or PostgreSQL publisher connectivity;
- Kafka delivery;
- the MQTT event loop.

Do not use it as a strict readiness signal.

## Metrics

Enable:

```toml
[gateway]
metrics_enabled = true
```

The `/metrics` endpoint exposes event counts and handling duration. It remains
public when JWT is enabled so monitoring systems do not need ingestion
credentials. Restrict metrics exposure at the network or ingress layer.

## Logs

Configure logging with `RUST_LOG`:

```bash
RUST_LOG=info event-gateway
RUST_LOG=event_gateway=debug event-gateway
```

Configuration logging excludes database and publisher credentials.

## Shutdown

The process currently relies on runtime/process termination. There is no
explicit coordinated drain for in-flight HTTP requests or publisher tasks.
Allow the platform termination grace period to exceed publisher timeouts.
