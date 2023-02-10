FROM rust:1.66 as build

RUN curl https://sh.rustup.rs -sSf | sh -s -- --no-modify-path --default-toolchain none -y
RUN rustup component add rustfmt
RUN apt-get update && apt-get install -y clang libclang-dev

WORKDIR /usr/src

COPY . .
RUN cargo fetch

RUN cargo build --bin pulzaard --release && \
    mkdir -p /out && \
    mv target/release/pulzaard /out/pulzaard

FROM debian:bullseye-slim as runtime
WORKDIR /pulzaar
COPY --from=build /out/pulzaard /usr/bin/pulzaard
ENV RUST_LOG=warn,pulzaard=info,pulzaar=info
CMD [ "/usr/bin/pulzaard" ]
