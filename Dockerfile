FROM rust:latest

WORKDIR /src
COPY . .

RUN cargo build --release
EXPOSE 32500
CMD [ "cargo", "run", "--release" ]