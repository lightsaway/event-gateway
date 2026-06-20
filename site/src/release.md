# Release Process

Releases are tag-driven.

## Prepare

Run the release checks before creating the version commit and tag:

```bash
make ci-check
make docker-build
```

Then bump the patch version, commit `Cargo.toml` and `Cargo.lock`, and create
the matching local tag:

```bash
make create-next-tag
```

To choose a specific version instead:

```bash
make create-next-tag VERSION=0.2.0
```

## Publish

Push the generated commit and tag:

```bash
git push origin HEAD v0.2.0
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
