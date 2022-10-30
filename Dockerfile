FROM rust:1.61 as builder
COPY . .
RUN cargo build --package hattrick_server --release

FROM debian:buster-slim
COPY --from=builder /target/release/hattrick_server ./target/release/hattrick_server
EXPOSE 8111
CMD ["./target/release/hattrick_server"]
