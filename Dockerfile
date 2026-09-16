# ==============================================================================
# Stage 1: Build static frontend (Svelte 5 + Vite) using pnpm
# ==============================================================================
FROM node:24-alpine AS frontend-builder
WORKDIR /app/frontend

RUN corepack enable && corepack prepare pnpm@9.2.0 --activate

COPY frontend/package.json frontend/pnpm-lock.yaml frontend/svelte.config.js* ./
RUN pnpm install --frozen-lockfile

COPY frontend ./
RUN pnpm run build

# ==============================================================================
# Stage 2: Build Rust backend (Axum server)
# ==============================================================================
FROM rust:alpine AS backend-builder
RUN apk add --no-cache musl-dev

WORKDIR /app/backend

# Pre-fetch & build dependencies for faster subsequent cached builds
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code and build production binary
COPY backend/src ./src
RUN touch src/main.rs && cargo build --release

# ==============================================================================
# Stage 3: Minimal Alpine production runtime
# ==============================================================================
FROM alpine:3.21 AS runner

RUN apk add --no-cache ca-certificates tzdata && \
    addgroup -S appgroup && adduser -S appuser -G appgroup

WORKDIR /app

# Copy binary from backend builder
COPY --from=backend-builder --chown=appuser:appgroup /app/backend/target/release/cosave /app/cosave

# Copy static frontend dist from frontend builder
COPY --from=frontend-builder --chown=appuser:appgroup /app/frontend/dist /app/dist

ENV STATIC_DIR=/app/dist
ENV PORT=3000

USER appuser
EXPOSE 3000

CMD ["/app/cosave"]
