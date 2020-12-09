FROM rust:slim-buster AS builder

COPY . /app
WORKDIR /app

RUN cargo build --release

FROM debian:buster-slim AS runner

ARG UID=999
ENV RUST_LOG=info

COPY --from=builder /app/target/release/glass-pane /usr/bin/glass-pane

RUN useradd -s /bin/false -r -u $UID glasspane && \
    mkdir -p /config && \
    chown -R glasspane:glasspane /config && \
    chmod 0755 /usr/bin/glass-pane

USER glasspane
EXPOSE 9946
ENTRYPOINT /usr/bin/glass-pane
CMD /config/glasspane.yml
