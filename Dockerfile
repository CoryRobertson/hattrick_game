FROM rust:1.66 as builder
COPY . .
#RUN apt-get update
#RUN apt-get install -y pkg-config libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
RUN cargo build --package hattrick_server --release

FROM debian:buster-slim
COPY --from=builder /target/release/hattrick_server ./target/release/hattrick_server
#RUN apt-get update
#RUN apt-get install -y pkg-config libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
EXPOSE 8111
CMD ["./target/release/hattrick_server"]
