FROM rust:1.83 as builder

WORKDIR /usr/src/mqtt-bench
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    make \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire project first
COPY . .

# Build the release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/mqtt-bench/target/release/mqtt-bench /app/mqtt-bench

ENTRYPOINT ["/app/mqtt-bench"]
