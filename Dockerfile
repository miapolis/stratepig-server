FROM rust:latest

WORKDIR /src
COPY . .

RUN cargo install --path .
EXPOSE 32500
CMD [ "cargo", "run", "--release" ]