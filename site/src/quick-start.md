# Quick Start

## Docker

Build and run the bundled no-op configuration:

```bash
docker build -t event-gateway:local .
docker run --rm -p 8080:8080 event-gateway:local
```

Check liveness:

```bash
curl http://localhost:8080/api/v1/health-check
```

The minimal image configuration has no routing rules, so event submissions are
rejected with HTTP 406 until a rule is created.

## Source

Requirements:

- Rust 1.88;
- Node.js 22;
- native C/C++ build tools for `librdkafka`.

```bash
git clone https://github.com/lightsaway/event-gateway.git
cd event-gateway
make frontend-build
cargo run --locked
```

By default the process loads `config.toml`. Set `APP_CONFIG_PATH` to use a
different file:

```bash
APP_CONFIG_PATH=./configs/config-pgmq.toml cargo run --locked
```

For a complete request and routing rule, continue to
[First Event](first-event.md).
