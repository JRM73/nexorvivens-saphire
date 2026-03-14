# Saphire — Multi-stage build
FROM rust:1.88-bookworm AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY src/ src/
COPY static/ static/
COPY prompts/ prompts/
COPY sql/ sql/
COPY config/factory_defaults.toml ./
COPY config/profiles/ profiles/
COPY config/personalities/ personalities/

RUN cargo build --release

# Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Utilisateur non-root pour la securite
RUN adduser --disabled-password --gecos '' saphire

WORKDIR /app

COPY --from=builder /build/target/release/saphire /app/saphire
RUN chown saphire:saphire /app/saphire

USER saphire

EXPOSE 3080

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -f http://localhost:3080/api/health || exit 1

ENTRYPOINT ["/app/saphire"]
CMD ["--config", "/app/saphire.toml"]
