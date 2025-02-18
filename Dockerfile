FROM bitnami/minideb:bookworm

RUN mkdir /app
WORKDIR /app
COPY target/release/tg-bot-rs /app
RUN install_packages openssl ca-certificates

ENTRYPOINT ["./tg-bot-rs"]
