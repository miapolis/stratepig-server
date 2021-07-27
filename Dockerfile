# 1: Build the exe
FROM rust:1.53.0-slim as builder
WORKDIR /usr

# 1a: Prepare for static linking
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./

# 1c: Build the exe using the actual source code
COPY stratepig_cli ./stratepig_cli
COPY stratepig_core ./stratepig_core
COPY stratepig_game ./stratepig_game
COPY stratepig_macros ./stratepig_macros
COPY stratepig_server ./stratepig_server
RUN cargo install --target x86_64-unknown-linux-musl --path .

# 2: Copy the exe and extra files ("static") to an empty Docker image
FROM scratch
COPY --from=builder /usr/local/cargo/bin/stratepig_server .
USER 1000
EXPOSE 32500
CMD ["./stratepig_server"]