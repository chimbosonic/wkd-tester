FROM rust:slim

RUN mkdir -p /build

COPY . /build

WORKDIR /build/server

RUN cargo build --release

FROM rust:slim

RUN mkdir -p /opt

COPY --from=0 /build/target/release/wkd-tester-server /opt/wkd-tester-server

COPY --from=0 /build/server/static /opt/static

RUN chmod +x /opt/wkd-tester-server
WORKDIR /opt

EXPOSE 7070

ENTRYPOINT ["/opt/wkd-tester-server"]