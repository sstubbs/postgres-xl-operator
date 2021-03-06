FROM rust:1.40 as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .

FROM debian:10.2-slim
RUN apt-get update && apt-get install -y pkg-config libssl-dev
COPY --from=builder /usr/local/cargo/bin/postgres-xl-operator /usr/local/bin/postgres-xl-operator
CMD ["postgres-xl-operator"]