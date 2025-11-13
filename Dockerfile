# Use the official Rust image as the base image
FROM rust:1.75 as builder

# Set the working directory
WORKDIR /usr/src/kandil_code

# Copy the Cargo files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to allow cargo to download and build dependencies
RUN mkdir src
RUN echo "fn main() { println!(\"dummy\"); }" > src/main.rs

# Download and build dependencies
RUN cargo build --release
RUN rm src/main.rs

# Now copy the actual source code
COPY . .

# Build the application
RUN cargo build --release

# Create a smaller image for the final binary
FROM debian:bookworm-slim

# Install ca-certificates for HTTPS requests
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/kandil_code/target/release/kandil_code /usr/local/bin/kandil

# Create a non-root user for security
RUN groupadd -r kandil && useradd -r -g kandil kandil

# Change ownership of the binary
RUN chown kandil:kandil /usr/local/bin/kandil

# Switch to the non-root user
USER kandil

# Set the default command
CMD ["kandil", "--help"]