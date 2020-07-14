FROM rust:slim as phi-build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src

RUN USER=root cargo new --lib phi

WORKDIR /usr/src/phi

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > ./src/main.rs

COPY Cargo.toml Cargo.lock ./

RUN cargo build --release

COPY ./src ./src

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian10

COPY --from=phi-build /usr/local/cargo/bin/phi /usr/local/bin/phi

CMD ["phi"]
