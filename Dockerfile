FROM node:22-bookworm-slim AS css

WORKDIR /app

COPY src/static/css/input.css src/static/css/input.css
COPY src/templates src/templates

RUN npm install --no-save @tailwindcss/cli@4.3.3 tailwindcss@4.3.3 \
    && npx tailwindcss \
    -i src/static/css/input.css \
    -o src/static/css/site.css \
    --minify

FROM rust:1.85-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock build.rs ./
COPY content content
COPY src src
COPY --from=css /app/src/static/css/site.css src/static/css/site.css

RUN cargo build --release --locked

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/rust_site /usr/local/bin/rust_site
COPY --from=builder /app/src/static src/static

ENV RUST_LOG=info

CMD ["rust_site"]
