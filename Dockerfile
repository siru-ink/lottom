## Container 1: Compile the rust code using musl to prevent external dependencies
FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev

# Copy sources into container 1 for building
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY templates ./templates
COPY .sqlx ./.sqlx

RUN cargo build --release

# Container 2: Minimal output
FROM scratch

# Copy needed runtime sources
COPY --from=builder /target/release/lottom /lottom
COPY --from=builder /migrations /migrations
COPY --from=builder /templates /templates

# Metainfo Setup
ENTRYPOINT ["/lottom"]
EXPOSE 80
