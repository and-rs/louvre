run:
    #!/usr/bin/env bash
    tailwindcss -i src/static/css/input.css -o src/static/css/site.css --silent
    tailwindcss -i src/static/css/input.css -o src/static/css/site.css --watch --silent &
    tailwind=$!
    trap 'kill "$tailwind" 2>/dev/null' EXIT INT TERM
    cargo watch -d 0.2 -w src -w content -i src/static/css/site.css \
        -s "rustywind --write --output-css-file src/static/css/site.css src/templates && cargo run --features dev"

format:
    cargo fmt
    tailwindcss -i src/static/css/input.css -o src/static/css/site.css
    rustywind --write --output-css-file src/static/css/site.css src/templates
    biome format --write src/static/js/site.js src/static/css/input.css

check:
    cargo fmt --check
    tailwindcss -i src/static/css/input.css -o src/static/css/site.css
    rustywind --check-formatted --output-css-file src/static/css/site.css src/templates
    biome check src/static/js/site.js
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-features

hooks:
    prek install

build:
    tailwindcss -i src/static/css/input.css -o src/static/css/site.css
    rustywind --write --output-css-file src/static/css/site.css src/templates
    cargo build --release
