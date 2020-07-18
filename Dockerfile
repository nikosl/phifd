FROM rust:slim as phi-build

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV app="phifd"

WORKDIR /usr/src
RUN USER=root cargo new phifd

WORKDIR /usr/src/phifd
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > ./src/main.rs

COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY ./src ./src
RUN set -x\
        && find target/release -type f -name "$(echo "${app}" | tr '-' '_')*" -exec touch -t 200001010000 {} +\
        && cargo build --release

FROM gcr.io/distroless/cc-debian10

COPY ./static/index.html /opt/phifd/static/
COPY --from=phi-build /usr/src/phifd/target/release/phifd /opt/phifd/phifd

WORKDIR /opt/phifd/
ENTRYPOINT ["./phifd"]
