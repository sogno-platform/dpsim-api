FROM alpine:latest AS builder

RUN apk update

# ============
#    DPSim
# ===========

# Toolchain
RUN apk add openssl-dev rustup build-base
RUN rustup-init -y --default-toolchain stable
RUN source $HOME/.cargo/env && rustc --version
RUN mkdir -p /usr/src/dpsim-api
COPY Rocket.toml Cargo.toml Cargo.lock /usr/src/dpsim-api/
WORKDIR /usr/src/dpsim-api
RUN mkdir /usr/src/dpsim-api/src
RUN touch /usr/src/dpsim-api/src/lib.rs
# The cargo build here will build the dependencies
# in a preparatory step, reducing docker build times
RUN source $HOME/.cargo/env && cargo build -r
COPY src/ /usr/src/dpsim-api/src
COPY templates/ /usr/src/dpsim-api/templates
RUN rm /usr/src/dpsim-api/src/lib.rs
RUN source $HOME/.cargo/env && cargo build -r

FROM alpine:latest
COPY --from=builder /usr/src/dpsim-api/target/release/dpsim-api /usr/bin
COPY --from=builder /usr/src/dpsim-api/templates/ /usr/bin/templates/
COPY --from=builder /usr/src/dpsim-api/Rocket.toml /usr/bin/Rocket.toml
WORKDIR /usr/bin
EXPOSE 8000
CMD /usr/bin/dpsim-api

