## Container 1: Compile the rust code using musl to prevent external dependencies
FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev git

# Copy sources into container 1 for building
RUN mkdir /lottom
RUN mkdir /graka
RUN mkdir -p /uploads
COPY Cargo.toml Cargo.lock ./lottom/
COPY src ./lottom/src
COPY migrations ./lottom/migrations
COPY templates ./lottom/templates
COPY .sqlx ./lottom/.sqlx

WORKDIR /lottom
RUN cargo build --release

WORKDIR /graka
RUN git clone https://code.siru.ink/siru-ink/graka.git .
RUN cargo build --release

# Container 2: Minimal output
FROM scratch

# Copy needed runtime sources & empty directories
COPY --from=builder /lottom/target/release/lottom /lottom
COPY --from=builder /lottom/migrations /migrations
COPY --from=builder /lottom/templates /templates
COPY --from=builder /uploads /uploads
COPY --from=builder /graka/target/release/graka /graka

# Metainfo Setup
VOLUME ["/uploads"]
ENTRYPOINT ["/lottom"]
EXPOSE 80
ENV HEALTHCHECK_PORT=80
ENV HEALTHCHECK_PATH=healthcheck
HEALTHCHECK --start-period=10s CMD ["/graka"]
