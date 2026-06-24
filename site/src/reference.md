# Reference

## Server

| Key | Required | Description |
|---|---:|---|
| `server.host` | yes | bind IP address |
| `server.port` | yes | HTTP port |

## Gateway

| Key | Required | Description |
|---|---:|---|
| `gateway.metrics_enabled` | yes | expose `/metrics` |
| `gateway.publisher.type` | yes | `noOp`, `pgmq`, `kafka`, or `mqtt` |

## PGMQ publisher

| Key | Default | Description |
|---|---:|---|
| `connection_url` | required | PostgreSQL URL |
| `max_connections` | `10` | SQLx pool limit, must be greater than zero |
| `delay_seconds` | `0` | PGMQ visibility delay, cannot be negative |
| `group_metadata_field` | unset | Default event metadata field copied to `x-pgmq-group` |

## PostgreSQL configuration storage

| Key | Default | Description |
|---|---:|---|
| `username` | required | database user |
| `password` | required | database password |
| `endpoint` | required | `host` or `host:port` |
| `dbname` | `event_gateway` | database name |
| `cache_refresh_interval_secs` | `300` | cache refresh interval |

## API

| Key | Default | Description |
|---|---:|---|
| `api.prefix` | `/` | route prefix |
| `api.jwt_auth.jwks_url` | unset | enables JWT authorization for `POST /event` |
