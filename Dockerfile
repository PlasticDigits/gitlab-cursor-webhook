# syntax=docker/dockerfile:1

FROM rust:1.88-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p gchcontroller -p gchconfig

FROM debian:bookworm-slim AS runtime

ARG TERRAFORM_VERSION=1.9.8

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates wget unzip \
    && wget -q "https://releases.hashicorp.com/terraform/${TERRAFORM_VERSION}/terraform_${TERRAFORM_VERSION}_linux_amd64.zip" \
    && unzip "terraform_${TERRAFORM_VERSION}_linux_amd64.zip" -d /usr/local/bin \
    && rm "terraform_${TERRAFORM_VERSION}_linux_amd64.zip" \
    && apt-get purge -y unzip \
    && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/gchcontroller /usr/local/bin/gchcontroller
COPY --from=builder /app/target/release/gchconfig /usr/local/bin/gchconfig
COPY terraform /app/terraform
COPY templates /app/templates
COPY scripts/docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh

RUN chmod +x /usr/local/bin/docker-entrypoint.sh \
    && mkdir -p /var/lib/gch/jobs

ENV GCH_TERRAFORM_MODULE=/app/terraform/modules/agent-vm \
    GCH_CLOUD_INIT_TEMPLATE=/app/templates/cloud_init.yaml.tpl \
    GCH_DB_PATH=/var/lib/gch/gch.db \
    GCH_JOBS_DIR=/var/lib/gch/jobs \
    LISTEN_ADDR=0.0.0.0:8080 \
    PORT=8080

VOLUME /var/lib/gch

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD wget -q -O- http://127.0.0.1:8080/health || exit 1

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["gchcontroller"]
