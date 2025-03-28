# Builder
FROM rust:1 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runner
FROM debian:bookworm-slim as runner
RUN apt update
RUN apt install openssl ca-certificates -y

COPY --from=builder /app/target/release/built-in-hybrid-server /usr/local/bin/built-in-hybrid-server
COPY --from=builder /app/static /static

CMD ["built-in-hybrid-server"]
