# Release Process

Releases are tag-driven.

## Prepare

1. Update the package version in `Cargo.toml`.
2. Update `Cargo.lock`.
3. Run:

```bash
make ci-check
make docker-build
```

4. Commit and merge to `master`.

## Publish

Create a matching annotated tag:

```bash
git tag -a v0.2.0 -m "v0.2.0"
git push origin v0.2.0
```

The workflow rejects a tag that does not match the Cargo package version.

## Release outputs

- Linux x86_64 binary archive;
- Linux arm64 binary archive;
- macOS x86_64 binary archive;
- macOS arm64 binary archive;
- SHA-256 checksums;
- artifact attestations;
- multi-architecture GHCR image;
- generated GitHub release notes.

Stable semantic-version tags also update `latest`.

## Permissions and secrets

No repository secrets are required for GHCR or GitHub Releases. The workflow
uses `GITHUB_TOKEN` with scoped `packages`, `attestations`, and `contents`
permissions.
