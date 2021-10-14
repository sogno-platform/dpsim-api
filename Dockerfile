FROM alpine:latest

RUN apk update

# ============
#    DPSim
# ===========

# Toolchain
RUN apk add openssl-dev rustup build-base
RUN rustup-init -y --default-toolchain nightly
RUN source $HOME/.cargo/env && rustc --version

RUN mkdir -p /usr/src/app
WORKDIR /usr/src/app
COPY . /usr/src/app
RUN source $HOME/.cargo/env && cargo build

EXPOSE 8000

CMD ip addr && $HOME/.cargo/bin/cargo run
