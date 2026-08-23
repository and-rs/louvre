FROM node:22-bookworm-slim AS css

WORKDIR /app

COPY src/static/css/input.css src/static/css/input.css
COPY src/templates src/templates

RUN npm install --no-save @tailwindcss/cli@4.3.3 tailwindcss@4.3.3 \
    && npx tailwindcss \
    -i src/static/css/input.css \
    -o src/static/css/site.css \
    --minify \
    && node -e "const z=require('zlib'),f=require('fs');f.writeFileSync('src/static/css/site.css.br',z.brotliCompressSync(f.readFileSync('src/static/css/site.css'),{params:{[z.constants.BROTLI_PARAM_QUALITY]:11}}))"

FROM rust:1.94-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock build.rs ./
COPY src src
COPY --from=css /app/src/static/css/site.css src/static/css/site.css

RUN cargo build --release --locked

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/sanarte /usr/local/bin/sanarte
COPY --from=builder /app/src/static src/static

ENV RUST_LOG=info

CMD ["sanarte"]
