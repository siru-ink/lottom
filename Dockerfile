# Build stage
FROM rust:alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create app directory
# WORKDIR /app

# CCopy everything needed into the builder container
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY templates ./templates
COPY .sqlx ./.sqlx

# Build the actual application
RUN cargo build --release

# Final stage - scratch
FROM scratch

# Copy the compiled binary
COPY --from=builder /target/release/lottom /lottom

# Copy migrations and templates
COPY --from=builder /migrations /migrations
COPY --from=builder /templates /templates

# Copy CA certificates if your app makes HTTPS requests
# COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Set the entrypoint
ENTRYPOINT ["/lottom"]
EXPOSE 80
