run:
    #!/usr/bin/env bash
    tailwindcss -i src/static/css/input.css -o src/static/css/site.css --silent
    tailwindcss -i src/static/css/input.css -o src/static/css/site.css --watch --silent &
    tailwind=$!
    trap 'kill "$tailwind" 2>/dev/null' EXIT INT TERM
    cargo watch -d 0.2 -w src -i src/static/css/site.css \
        -s "rustywind --write --output-css-file src/static/css/site.css src/templates && cargo run --features dev"

format:
    cargo fmt
    tailwindcss -i src/static/css/input.css -o src/static/css/site.css
    rustywind --write --output-css-file src/static/css/site.css src/templates
    biome format --write src/static/js/dev.js src/static/js/site.js src/static/css/input.css

check:
    cargo fmt --check
    tailwindcss -i src/static/css/input.css -o src/static/css/site.css
    just compress
    rustywind --check-formatted --output-css-file src/static/css/site.css src/templates
    biome check src/static/js/dev.js src/static/js/site.js
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-features

compress:
    #!/usr/bin/env bash
    set -euo pipefail
    for f in src/static/css/site.css src/static/js/site.js src/static/js/mu.min.js; do
        brotli -q 11 -f "$f" -o "$f.br"
        printf '%s: %s -> %s\n' "$f" "$(du -h "$f" | cut -f1)" "$(du -h "$f.br" | cut -f1)"
    done

hooks:
    prek install

icon name:
    ./scripts/icon.sh {{name}}

build:
    tailwindcss -i src/static/css/input.css -o src/static/css/site.css
    rustywind --write --output-css-file src/static/css/site.css src/templates
    just compress
    cargo build --release
