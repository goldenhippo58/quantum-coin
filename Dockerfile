# Stage 1: Builder
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    libclang-dev \
    clang \
    file

# Clean up apt cache
RUN rm -rf /var/lib/apt/lists/*

# Set LIBCLANG_PATH for bindgen
ENV LIBCLANG_PATH=/usr/lib/llvm-14/lib

WORKDIR /app

# Copy the entire project
COPY . .

# Set up library and include paths
RUN mkdir -p /usr/local/lib
RUN cp libs/libpqcrystals_*.so /usr/local/lib/
RUN cp libs/libsphincsshake128f.so /usr/local/lib/
RUN ldconfig

# Debug: Print contents to verify files
RUN ls -R include/
RUN ls -R libs/
RUN cat wrapper.h

# Build the project
RUN LIBRARY_PATH=/usr/local/lib \
    LD_LIBRARY_PATH=/usr/local/lib \
    CFLAGS="-I$(pwd)/include" \
    cargo build --release

# Debug: Print binary information
RUN ls -la /app/target/release/
RUN pwd && ls -la /app/target
RUN file /app/target/release/quantumcoin || true

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    file
RUN rm -rf /var/lib/apt/lists/*

# Create necessary directories
RUN mkdir -p /usr/local/lib /usr/local/bin /app

# Copy binaries and libraries
COPY --from=builder /usr/local/lib/libpqcrystals_*.so /usr/local/lib/
COPY --from=builder /usr/local/lib/libsphincsshake128f.so /usr/local/lib/
COPY --from=builder /app/target/release/quantumcoin /usr/local/bin/quantumcoin

# Debug: Verify binary
RUN ls -la /usr/local/bin/quantumcoin
RUN file /usr/local/bin/quantumcoin || true

# Make binary executable
RUN chmod +x /usr/local/bin/quantumcoin

# Copy include files
COPY --from=builder /app/include/kyber /usr/local/include/kyber
COPY --from=builder /app/include/sphincs /usr/local/include/sphincs

# Update library cache
RUN ldconfig

WORKDIR /app

# Use ENTRYPOINT to ensure the binary is always executed
ENTRYPOINT ["/usr/local/bin/quantumcoin"]
CMD []