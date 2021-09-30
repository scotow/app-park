FROM rust:1.54-slim AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

#------------

FROM debian:buster-slim

COPY --from=builder /app/target/release/app-park /app-park

ENTRYPOINT ["/app-park"]