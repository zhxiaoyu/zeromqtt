# Build stage
FROM rust:1.75-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev pkgconfig zeromq-dev nodejs npm

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY dashboard/package.json dashboard/

# Build dependencies (for caching)
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release || true

# Copy source
COPY . .

# Build frontend
RUN cd dashboard && npm install && npm run build

# Build Rust binary
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache libgcc openssl zeromq

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/zeromqtt /app/zeromqtt

# Create data directory
RUN mkdir -p /root/.zeromqtt

# Expose ports
EXPOSE 3000

# Environment variables
ENV RUST_LOG=info
ENV ZEROMQTT_PORT=3000

CMD ["/app/zeromqtt"]
