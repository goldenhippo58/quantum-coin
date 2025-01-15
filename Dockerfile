# Stage 1: Build stage
FROM rust:latest AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    build-essential \
    librocksdb-dev \
    pkg-config \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy source files
COPY . .

# Build the project
RUN cargo build --release

# Stage 2: Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    librocksdb-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/quantumcoin .

# Expose necessary ports
EXPOSE 8080

# Set the default command to run the binary
CMD ["./quantumcoin"]
