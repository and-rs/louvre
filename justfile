run:
    @bash -c 'tailwindcss -i src/static/css/input.css -o src/static/css/site.css --watch & css_pid=$!; trap "kill $$css_pid" EXIT INT TERM; cargo watch -w src -w content -x "run --features dev"'

css:
    tailwindcss -i src/static/css/input.css -o src/static/css/site.css --minify

format:
    cargo fmt
    biome format --write src/static/js/site.js src/static/css/input.css

check:
    cargo fmt --check
    biome check src/static/js/site.js src/static/css/input.css
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-features

hooks:
    prek install

build: css
    cargo build --release
