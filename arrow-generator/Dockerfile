# 1. This tells docker to use the Rust official image
FROM rust:1-slim-bookworm

# 2. Copy the files in your machine to the Docker image
COPY ./ ./

# Build your program for release
# --release
RUN cargo build

# Run the binary
# CMD ["./target/release/arrow-generator"]
ENTRYPOINT ["./target/debug/arrow-generator"]