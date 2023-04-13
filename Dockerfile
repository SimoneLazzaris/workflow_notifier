FROM rust:1.68.2-slim-bullseye as builder
RUN apt-get update && apt-get install -y libssl-dev pkg-config
WORKDIR /app
COPY Cargo.toml .
COPY src /app/src
RUN cargo build -r

FROM debian:11-slim as target
COPY --from=builder /app/target/release/workflow_notifier /usr/local/bin/workflow_notifier
ENTRYPOINT ["/usr/local/bin/workflow_notifier"]
