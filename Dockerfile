FROM rust:slim-buster AS builder

ARG CARGO_FLAGS="--features=commands"

COPY . /app
WORKDIR /app

RUN apt-get update && \
    apt-get install -y libssl-dev pkg-config

RUN cargo build --release

FROM debian:buster-slim AS runner

ARG UID=999
ENV RUST_LOG=info

COPY --from=builder /app/target/release/glass-pane /usr/bin/glass-pane

RUN apt-get update && \
    apt-get install -y libssl1.0 ca-certificates && \
    useradd -s /bin/false -r -u $UID glass-pane && \
    chmod 0755 /usr/bin/glass-pane

USER glass-pane
EXPOSE 9946
ENTRYPOINT /usr/bin/glass-pane /config.yml
