FROM rust:1.75-alpine as builder

# Install build dependencies
RUN apk add --no-cache \
    pkgconfig \
    musl-dev \
    cmake \
    make \
    gcc \
    g++ \
    libressl-dev

# Create a new empty shell project
WORKDIR /usr/src/event-gateway
COPY . .

# Build the application with release optimizations
RUN cargo build --release

# Create a new stage with a minimal image
FROM alpine:3.16

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libressl

# Copy the build artifact from the builder stage
COPY --from=builder /usr/src/event-gateway/target/release/event-gateway /usr/local/bin/event-gateway
COPY --from=builder /usr/src/event-gateway/config.toml /etc/event-gateway/config.toml
COPY --from=builder /usr/src/event-gateway/config-postgres.toml /etc/event-gateway/config-postgres.toml

# Create a non-root user to run the application
RUN adduser -D -u 1000 event-gateway
USER event-gateway

# Set the working directory
WORKDIR /home/event-gateway

# Expose the port the app runs on
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV APP_CONFIG_PATH=/etc/event-gateway/config.toml

# Run the application
CMD ["event-gateway"] 