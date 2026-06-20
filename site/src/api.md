# HTTP API

All paths are nested under `api.prefix`, `/api/v1` in the examples.

| Method | Path | Purpose |
|---|---|---|
| POST | `/event` | validate, route, and publish an event |
| GET | `/routing-rules` | list routing rules |
| POST | `/routing-rules` | create a routing rule |
| PUT | `/routing-rules/:id` | replace a routing rule |
| DELETE | `/routing-rules/:id` | delete a routing rule |
| GET | `/topic-validations` | list validations grouped by topic |
| POST | `/topic-validations` | create a validation |
| DELETE | `/topic-validations/:id` | delete a validation |
| GET | `/health-check` | process liveness |
| GET | `/metrics` | Prometheus metrics, when enabled |

## Event responses

| Status | Meaning |
|---|---|
| 200 | publisher reported success |
| 400 | schema validation failed or request was invalid |
| 406 | no routing rule matched |
| 500 | storage or publisher failure |

## Request metadata

The gateway adds transport metadata:

- JWT `sub` and `iss`, when present;
- `x-forwarded-for` or `x-real-ip`;
- `user-agent`.

Caller-provided `transportMetadata` is replaced.

## Authentication

When `api.jwt_auth` is configured, the JWT authorizer wraps all API routes.
Operational probes must then send a valid token or use a separate deployment
strategy.
