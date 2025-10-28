FROM rust:1.70-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    git \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Create data directory
RUN mkdir -p /app/data

# Expose ports
EXPOSE 3001 9735

# Start the node
CMD ["./target/release/rgb-lightning-node", "/app/data", "--daemon-listening-port", "3001", "--ldk-peer-listening-port", "9735", "--network", "testnet"]