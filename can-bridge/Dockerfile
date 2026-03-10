FROM --platform=$BUILDPLATFORM rust:slim AS builder
RUN apt-get update && apt-get install -y gcc-aarch64-linux-gnu
RUN rustup target add aarch64-unknown-linux-gnu
WORKDIR /app
COPY . .
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
RUN cargo build --release --target aarch64-unknown-linux-gnu

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/aarch64-unknown-linux-gnu/release/can-bridge /usr/local/bin/can-bridge
CMD ["can-bridge"]
