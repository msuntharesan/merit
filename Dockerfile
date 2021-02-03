# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:1.49-alpine3.12 as toolchain

RUN adduser -D -h /merit -g "" merit

RUN apk add make zlib-dev openssl-dev musl-dev

USER merit

ENV USER=merit
ENV PATH=/merit/.cargo/bin:$PATH

WORKDIR /merit

RUN cargo init --lib --vcs none
RUN cargo new merit --lib --vcs none
RUN cargo new humanize --lib --vcs none
RUN cargo new merit-web --lib --vcs none

RUN mkdir .cargo
RUN cargo vendor > .cargo/config

ADD . .

RUN OPENSSL_STATIC=true \
    cargo build --release --target=x86_64-unknown-linux-musl -p merit-web

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:3.12

ENV PORT=8080 \
    LOG_LEVEL="actix_web=info"

WORKDIR /home/merit/bin/

COPY --from=toolchain /merit/target/x86_64-unknown-linux-musl/release/merit-web .

RUN adduser -D -H -g "" merit

USER merit

EXPOSE ${PORT}

CMD ["./merit-api"]