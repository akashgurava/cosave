# ==============================================================================
# Stage 1: Build static frontend (Svelte 5 + Vite) using dev.sh
# ==============================================================================
FROM node:24-alpine AS frontend-builder
RUN apk add --no-cache bash
WORKDIR /app

RUN corepack enable && corepack prepare pnpm@9.2.0 --activate

COPY dev.sh ./
COPY scripts/dev ./scripts/dev
COPY frontend/package.json frontend/pnpm-lock.yaml frontend/svelte.config.js* ./frontend/
RUN ./dev.sh ui pnpm install --frozen-lockfile

COPY frontend ./frontend
RUN ./dev.sh ui build

# ==============================================================================
# Stage 2: Build Rust backend (Axum server) using dev.sh
# ==============================================================================
FROM rust:alpine AS backend-builder
RUN apk add --no-cache musl-dev bash
WORKDIR /app

COPY dev.sh ./
COPY scripts/dev ./scripts/dev

# Pre-fetch & build dependencies for faster subsequent cached builds
COPY backend/Cargo.toml backend/Cargo.lock ./backend/
RUN mkdir -p backend/src && \
    touch backend/src/lib.rs && \
    echo "fn main() {}" > backend/src/main.rs && \
    ./dev.sh backend build --release && \
    rm -rf backend/src

# Copy actual source code and build production binary
COPY backend/src ./backend/src
RUN touch backend/src/main.rs && ./dev.sh backend build --release

# ==============================================================================
# Stage 3: Minimal Alpine production runtime
# ==============================================================================
FROM alpine:3.21 AS runner

RUN apk add --no-cache ca-certificates tzdata sqlite-libs && \
    addgroup -g 1000 -S appgroup && adduser -u 1000 -S appuser -G appgroup

WORKDIR /app

# Copy binary from backend builder
COPY --from=backend-builder --chown=appuser:appgroup /app/backend/target/release/cosave /app/cosave

# Copy static frontend dist from frontend builder
COPY --from=frontend-builder --chown=appuser:appgroup /app/frontend/dist /app/dist

# Create persistent data directory with non-root ownership
RUN mkdir -p /app/data && chown -R appuser:appgroup /app/data

ENV COSAVE_ENV=PROD
ENV COSAVE_HOST=0.0.0.0
ENV COSAVE_PORT=5172
ENV COSAVE_STATIC_DIR=/app/dist
ENV DATABASE_URL="sqlite:///app/data/cosave.db?mode=rwc"

VOLUME ["/app/data"]

USER appuser
EXPOSE 5172

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://127.0.0.1:5172/api/v1/health || exit 1

ENTRYPOINT ["/app/cosave"]
CMD []
