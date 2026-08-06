run:
    @bash -c 'cargo watch -w src -w content -s "cd /home/and-rs/Vault/dev/rust-site && tailwindcss -i src/static/css/input.css -o src/static/css/site.css && cargo run --features dev"'

format:
    cargo fmt
    biome format --write src/static/js/site.js src/static/css/input.css

check:
    cargo fmt --check
    biome check src/static/js/site.js
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-features

hooks:
    prek install

build:
    tailwindcss -i src/static/css/input.css -o src/static/css/site.css
    cargo build --release
