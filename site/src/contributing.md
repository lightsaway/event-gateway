# Contributing

## Toolchain

- Rust 1.88;
- Node.js 22;
- Docker with BuildKit;
- mdBook 0.5.3;
- lychee 0.24.2.

## Local checks

```bash
make ci-quality
make ci-test
make ci-ui
make ci-docs
make ci-audit
```

Run the full deterministic set:

```bash
make ci-check
```

## Documentation

Build and link-check:

```bash
make site-check
```

Serve locally:

```bash
make site-serve
```

Documentation must describe current behavior. Avoid diagrams that require
runtime plugins; use text diagrams so GitHub Pages and local mdBook output are
consistent.
