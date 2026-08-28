FROM node:22-alpine AS web-builder
WORKDIR /build
COPY package.json package-lock.json* ./
RUN npm ci
COPY index.html tsconfig.json vite.config.ts ./
COPY src ./src
COPY public ./public
RUN npm run build:web

FROM rust:1.98-alpine AS api-builder
ARG BUILD_SHA=dev
ENV BUILD_SHA=${BUILD_SHA}
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY server/Cargo.toml server/Cargo.lock* ./
COPY server/src ./src
COPY server/migrations ./migrations
RUN cargo build --release --locked

FROM alpine:3.22 AS runtime
ARG BUILD_SHA=dev
RUN addgroup -S app && adduser -S -G app app \
    && mkdir -p /app/dist /data \
    && chown -R app:app /app /data
COPY --from=api-builder /build/target/release/client-action-room-api /usr/local/bin/client-action-room-api
COPY --from=web-builder /build/dist /app/dist
WORKDIR /app
USER app
ENV PORT=8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/client-action-room-api"]
