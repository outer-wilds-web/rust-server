FROM rust:latest

WORKDIR /app

# Install system dependencies required for rdkafka
RUN apt-get update && apt-get install -y \
    cmake \
    pkg-config \
    libssl-dev \
    libsasl2-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application
RUN cargo build --release

# Run the binary
CMD ["./target/release/solar-sytem-simulation"]