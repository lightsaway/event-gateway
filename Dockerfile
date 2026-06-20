# syntax=docker/dockerfile:1.7

FROM node:22-alpine AS ui-builder

WORKDIR /app/ui
COPY ui/package.json ui/package-lock.json ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY ui ./
RUN npm run build

FROM rust:1.96-alpine AS builder

RUN apk add --no-cache \
    pkgconfig \
    musl-dev \
    cmake \
    make \
    gcc \
    g++ \
    openssl-dev \
    openssl-libs-static \
    zlib-static \
    curl-dev \
    perl \
    linux-headers

WORKDIR /usr/src/event-gateway
ENV OPENSSL_STATIC=1

COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY loadtest/Cargo.toml loadtest/Cargo.toml
COPY migrations ./migrations
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/event-gateway/target \
    mkdir -p src loadtest/src ui/dist \
    && printf 'fn main() {}\n' > src/main.rs \
    && printf 'fn main() {}\n' > loadtest/src/main.rs \
    && printf '<!doctype html>\n' > ui/dist/index.html \
    && cargo build --release --locked \
    && rm -rf src loadtest/src ui

COPY src ./src
COPY loadtest/src ./loadtest/src
COPY --from=ui-builder /app/ui/dist ./ui/dist

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/event-gateway/target \
    cargo clean --release -p event-gateway \
    && cargo build --release --locked -p event-gateway \
    && cp target/release/event-gateway /tmp/event-gateway

FROM alpine:3.21

RUN apk add --no-cache ca-certificates libgcc

COPY --from=builder /tmp/event-gateway /usr/local/bin/event-gateway

RUN adduser -D -u 1000 event-gateway \
    && mkdir -p /etc/event-gateway \
    && chown -R event-gateway:event-gateway /etc/event-gateway

COPY configs/minimal-config.toml /etc/event-gateway/config.toml

USER event-gateway
WORKDIR /home/event-gateway

EXPOSE 8080
ENV RUST_LOG=info
ENV APP_CONFIG_PATH=/etc/event-gateway/config.toml

HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD wget -q --spider http://127.0.0.1:8080/api/v1/health-check || exit 1

CMD ["event-gateway"]
