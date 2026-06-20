# Configuration

Set the file path with `APP_CONFIG_PATH`. The default is `config`.

```toml
debug_mode = false

[server]
host = "0.0.0.0"
port = 8080

[database]
type = "inMemory"

[gateway]
metrics_enabled = true

[gateway.publisher]
type = "noOp"

[api]
prefix = "/api/v1"
```

## Environment overrides

Use `APP_` and double underscores between nested sections. Keep underscores
inside field names:

```bash
APP_SERVER__PORT=8081
APP_DATABASE__CACHE_REFRESH_INTERVAL_SECS=60
APP_GATEWAY__PUBLISHER__CONNECTION_URL=postgres://user:pass@db:5432/app
```

Comma-separated values are supported for Kafka brokers:

```bash
APP_GATEWAY__PUBLISHER__BROKERS=kafka-1:9092,kafka-2:9092
```

## Configuration storage

### In memory

```toml
[database]
type = "inMemory"
initial_data_json = '''{
  "routing_rules": [],
  "topic_validations": {}
}'''
```

Changes are lost on restart.

### File

```toml
[database]
type = "file"
path = "/var/lib/event-gateway"
```

Routing rules and validations are persisted as JSON files.

### PostgreSQL

```toml
[database]
type = "postgres"
endpoint = "postgres:5432"
username = "event_gateway"
password = "secret"
dbname = "event_gateway"
cache_refresh_interval_secs = 300
```

Migrations run during startup. Startup fails if PostgreSQL is unavailable or
the initial cache cannot be loaded.

## JWT

```toml
[api]
prefix = "/api/v1"

[api.jwt_auth]
jwks_url = "https://issuer.example/.well-known/jwks.json"
```

When configured, JWT authorization applies to every nested API route,
including health and metrics.
