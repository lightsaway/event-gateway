# Event Gateway

A high-performance event routing and validation service that acts as a central hub for event-driven architectures.

## Overview

Event Gateway is a service that receives events, validates them against predefined schemas, and routes them to appropriate topics based on configurable rules. It supports multiple storage backends (in-memory and PostgreSQL) and provides a REST API for event submission and configuration management.

## Features

- **Event Routing**: Route events to topics based on event type and version
- **Schema Validation**: Validate event data against JSON schemas
- **Multiple Storage Backends**: Support for in-memory and PostgreSQL storage
- **REST API**: HTTP endpoints for event submission and configuration
- **Database Migrations**: Automatic database schema migrations
- **Caching**: In-memory caching for routing rules and topic validations
- **Configurable**: Extensive configuration options via TOML files

## Getting Started

### Prerequisites

- Rust 1.70+ (for development)
- PostgreSQL 13+ (for PostgreSQL storage backend)
- Docker and Docker Compose (for containerized deployment)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/event-gateway.git
   cd event-gateway
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

### Configuration

Event Gateway is configured using TOML files. Two configuration files are provided:

- `config.toml`: Default configuration with in-memory storage
- `config-postgres.toml`: Configuration for PostgreSQL storage

Example configuration:

```toml
[server]
host = "0.0.0.0"
port = 8080
api_prefix = "/api/v1"

[storage]
type = "postgres"  # or "memory"

[storage.postgres]
endpoint = "localhost:5432"
username = "postgres"
password = "postgres"
dbname = "event_gateway"
cache_refresh_interval_secs = 300
```

### Running the Service

#### Development Mode

```bash
cargo run
```

#### Production Mode

```bash
cargo build --release
./target/release/event-gateway
```

#### With PostgreSQL

```bash
APP_CONFIG_PATH=config-postgres.toml cargo run
```

### Docker Deployment

The project includes Docker and Docker Compose configuration for containerized deployment:

#### Building the Docker Image

```bash
docker build -t event-gateway .
```

#### Running with Docker Compose

```bash
# Start all services
docker-compose up -d

# Stop all services
docker-compose down
```

The Docker Compose configuration sets up:
- Event Gateway service
- PostgreSQL database
- Networking between services
- Persistent volume for PostgreSQL data

### Infrastructure Setup

The project includes Docker Compose configuration for setting up the required infrastructure:

```bash
# Start infrastructure
make infra-run

# Stop infrastructure
make infra-stop
```

## API Usage

### Sending Events

Events can be sent to the gateway using the `/event` endpoint:

```bash
curl -X POST http://localhost:8080/api/v1/event \
  -H "Content-Type: application/json" \
  -d '{
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "eventType": "user.created",
    "eventVersion": "1.0",
    "metadata": {
      "source": "test-script",
      "environment": "test"
    },
    "data": {
      "type": "json",
      "content": {
        "id": "user-123",
        "name": "John Doe",
        "email": "john@example.com",
        "created_at": "2023-01-01T12:00:00Z"
      }
    }
  }'
```

### Managing Routing Rules

#### Get All Routing Rules

```bash
curl -X GET http://localhost:8080/api/v1/routing-rules
```

#### Create a Routing Rule

```bash
curl -X POST http://localhost:8080/api/v1/routing-rules \
  -H "Content-Type: application/json" \
  -d '{
    "order": 0,
    "topic": "user-events",
    "description": "Route user events to user-events topic",
    "eventTypeCondition": {
      "type": "equals",
      "value": "user.created"
    },
    "eventVersionCondition": {
      "type": "equals",
      "value": "1.0"
    }
  }'
```

### Managing Topic Validations

#### Get All Topic Validations

```bash
curl -X GET http://localhost:8080/api/v1/topic-validations
```

#### Create a Topic Validation

```bash
curl -X POST http://localhost:8080/api/v1/topic-validations \
  -H "Content-Type: application/json" \
  -d '{
    "topic": "user-events",
    "schemas": [
      {
        "type": "json",
        "schema": {
          "type": "object",
          "properties": {
            "id": { "type": "string" },
            "name": { "type": "string" },
            "email": { "type": "string", "format": "email" },
            "created_at": { "type": "string", "format": "date-time" }
          },
          "required": ["id", "name", "email", "created_at"]
        }
      }
    ]
  }'
```

## Testing

### Running Tests

```bash
cargo test
```

### Load Testing

The project includes a load testing tool:

```bash
make loadtest
```

## Scripts

The `scripts` directory contains utility scripts for interacting with the service:

- `add_routing_rules.sh`: Add routing rules to the service
- `add_topic_validations.sh`: Add topic validations to the service
- `send_test_events.sh`: Send test events to the service
- `setup_database.sh`: Master script to set up the database with rules and validations

Example usage:

```bash
# Send test events
./scripts/send_test_events.sh http://localhost:8080/api/v1

# Set up database with rules and validations
./scripts/setup_database.sh http://localhost:8080/api/v1
```

## Architecture

Event Gateway follows a modular architecture:

- **HTTP Layer**: Handles incoming HTTP requests
- **Gateway**: Core service that processes events
- **Router**: Routes events to topics based on rules
- **Storage**: Manages routing rules and topic validations
- **Validator**: Validates event data against schemas

## TODO

- enable BASE_PATH propagation to FE during serving
- make UI page for showing event samples
- plugin metered backend for prometheus
- test MQTT routing
- add RabbitMQ publisher
- add jwt secret configuration option
- add auth header or separate endpoint to test event
- test event routing should also return topic

## License

This project is licensed under the MIT License - see the LICENSE file for details. 