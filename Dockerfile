FROM ubuntu:20.04 as builder

RUN apt-get update \
    && apt-get install -y curl g++-aarch64-linux-gnu libc6-dev-arm64-cross \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup target add aarch64-unknown-linux-gnu 
RUN rustup toolchain install stable-aarch64-unknown-linux-gnu

WORKDIR /app

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

COPY Cargo.toml Cargo.lock ./
COPY ./src ./src 
RUN cargo build --release --target aarch64-unknown-linux-gnu

FROM ubuntu:20.04
COPY --from=builder /app/target/aarch64-unknown-linux-gnu/release/http-server-rust /bin/http-server-rust
EXPOSE 4222 4222
CMD ["/bin/http-server-rust"]