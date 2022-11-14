FROM balenalib/raspberrypi4-64-debian AS builder1
SHELL ["/bin/bash", "-c"]
RUN --mount=type=cache,target=/var/cache/apt \
  apt-get update \
  && apt-get install -y libdbus-1-dev curl build-essential \
  && curl https://sh.rustup.rs -sSf | sh -s -- -y

FROM builder1 as builder2
SHELL ["/bin/bash", "-c"]
ENV PATH=/root/.cargo/bin:$PATH
WORKDIR /usr/src/app
COPY Cargo.toml .
COPY Cargo.lock .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  mkdir src \
  && echo "fn main() {}" > src/main.rs \
  && cargo fetch \
  && rm -f src/main.rs

FROM builder2 as builder3
SHELL ["/bin/bash", "-c"]
ENV PATH=/root/.cargo/bin:$PATH
WORKDIR /usr/src/app
COPY src ./src
COPY Cargo.toml .
COPY Cargo.lock .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/src/app/target \
  cargo build --release \
  && tar -czf build.tar.gz -C /usr/src/app/target/release ./ember-network-connect

FROM scratch AS export-stage
COPY --from=builder3 /usr/src/app/build.tar.gz /

