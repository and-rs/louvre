FROM node:22-bookworm-slim AS css

WORKDIR /app

COPY louvre-site/src/static/css/input.css louvre-site/src/static/css/input.css
COPY louvre-site/src/templates louvre-site/src/templates

RUN npm install --no-save @tailwindcss/cli@4.3.3 tailwindcss@4.3.3 \
    && npx tailwindcss \
    -i louvre-site/src/static/css/input.css \
    -o louvre-site/src/static/css/site.css \
    --minify \
    && node -e "const z=require('zlib'),f=require('fs');f.writeFileSync('louvre-site/src/static/css/site.css.br',z.brotliCompressSync(f.readFileSync('louvre-site/src/static/css/site.css'),{params:{[z.constants.BROTLI_PARAM_QUALITY]:11}}))"

FROM rust:1.94-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY louvre-site/Cargo.toml louvre-site/Cargo.toml
COPY louvre-tw-merge/Cargo.toml louvre-tw-merge/Cargo.toml
COPY louvre-site louvre-site
COPY louvre-tw-merge louvre-tw-merge
COPY --from=css /app/louvre-site/src/static/css/site.css louvre-site/src/static/css/site.css
COPY --from=css /app/louvre-site/src/static/css/site.css.br louvre-site/src/static/css/site.css.br

RUN cargo build -p louvre-site --bin louvre --release --locked

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/louvre /usr/local/bin/louvre
COPY --from=builder /app/louvre-site/src/static louvre-site/src/static

ENV RUST_LOG=info

CMD ["louvre"]
