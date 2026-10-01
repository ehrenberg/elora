# Container für den Elora-Master-Server (M8.4, E-162).
# Bauen (im Projektordner):  docker build -f deploy/master/Dockerfile -t elora-master .
# Starten:                    docker run -d --restart unless-stopped -p 127.0.0.1:8300:8300 elora-master
# HTTPS macht ein Reverse-Proxy davor (siehe docs/15-master-betrieb.md).
FROM rust:1.98-slim AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p elora-master

FROM debian:bookworm-slim
RUN useradd --system --no-create-home elora-master
COPY --from=build /src/target/release/elora-master /usr/local/bin/elora-master
USER elora-master
EXPOSE 8300
ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/elora-master", "--bind", "0.0.0.0:8300", "--behind-proxy"]
