FROM rust:slim AS build

RUN apt update && \ 
    apt install -y ca-certificates pkg-config libssl-dev gcc musl musl-tools libssl-dev build-essential libc6-dev && \ 
    apt upgrade -y

WORKDIR /build

COPY src /build/src
COPY Cargo.toml /build/Cargo.toml
COPY Cargo.lock /build/Cargo.lock

ENV RUSTFLAGS='-C target-feature=+crt-static'
ENV TARGET_CPU='native'

RUN --mount=type=cache,target=/usr/local/cargo/registry \ 
    --mount=type=cache,target=/root/.cargo/git \
    rustup target add x86_64-unknown-linux-musl 
    
RUN --mount=type=cache,target=/usr/local/cargo/registry \ 
    --mount=type=cache,target=/root/.cargo/git \
    cargo build --release --target x86_64-unknown-linux-musl

RUN strip /build/target/x86_64-unknown-linux-musl/release/nscn-match-maker

FROM scratch

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=build /build/target/x86_64-unknown-linux-musl/release/nscn-match-maker /nscn-match-maker

STOPSIGNAL SIGINT

CMD ["/nscn-match-maker"]