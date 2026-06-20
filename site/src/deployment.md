# Deployment

## Published image

Tagged releases publish:

```text
ghcr.io/lightsaway/event-gateway:<version>
ghcr.io/lightsaway/event-gateway:<major>.<minor>
ghcr.io/lightsaway/event-gateway:<major>
ghcr.io/lightsaway/event-gateway:latest
```

Run with a mounted configuration:

```bash
docker run --rm \
  -p 8080:8080 \
  -v "$PWD/config.toml:/etc/event-gateway/config.toml:ro" \
  ghcr.io/lightsaway/event-gateway:0.1.0
```

No Docker Hub token is required. Publishing uses GitHub's `GITHUB_TOKEN` and
GitHub Container Registry.

## Kubernetes probes

The image defines `/api/v1/health-check` as its Docker health check. For
Kubernetes, use it for liveness only. If JWT is enabled, the endpoint is also
protected and a plain HTTP probe will fail.

## Configuration secrets

Do not bake credentials into images. Mount a configuration file from a secret
or provide nested environment variables:

```bash
APP_GATEWAY__PUBLISHER__CONNECTION_URL=postgres://user:pass@db:5432/app
```

## Database roles

Use separate PostgreSQL roles where practical:

- Event Gateway needs permission to call `pgmq.send`;
- pgmq-relay needs read, archive, or delete permissions for its queues;
- PostgreSQL configuration storage needs migration and CRUD permissions.
