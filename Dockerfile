FROM rust:slim

RUN mkdir -p /build && apt-get update && apt-get install -y libssl-dev pkg-config

COPY . /build

WORKDIR /build/server

RUN cargo build --release

FROM rust:slim

COPY --from=0 /build/target/release/wkd-tester-server /usr/local/bin/wkd-tester-server
RUN chmod +x /usr/local/bin/wkd-tester-server

EXPOSE 7070

ENTRYPOINT ["/usr/local/bin/wkd-tester-server"]