# Build stage
FROM rust:latest as builder

WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/expenses_backend /app/

# Copy migrations
COPY --from=builder /usr/src/app/migrations ./migrations

# Expose port
EXPOSE 8080

# Run the application
CMD ["./expenses_backend"]
