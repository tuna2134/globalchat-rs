FROM rust:slim AS builder

WORKDIR /src/builder

ENV ARCH x86_64

RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add $ARCH-unknown-linux-musl

COPY . .
RUN --mount=type=cache,target=/src/builder/target/ cargo build --target=$ARCH-unknown-linux-musl --release && \
  cp target/$ARCH-unknown-linux-musl/release/globalchat-rs /tmp/globalchat-rs

FROM scratch

WORKDIR /src/app

COPY --from=builder /tmp/globalchat-rs .

CMD ["./hello"]