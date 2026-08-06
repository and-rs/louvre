run:
    @bash -c 'tailwindcss -i static/css/input.css -o static/css/site.css --watch & css_pid=$!; trap "kill $$css_pid" EXIT INT TERM; cargo watch -w src -w content -w static/js -w static/css/input.css -x "run --features dev"'

css:
    tailwindcss -i static/css/input.css -o static/css/site.css --minify

format:
    cargo fmt
    biome format --write static/js/site.js static/css/input.css

check:
    cargo fmt --check
    biome check static/js/site.js static/css/input.css
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-features

hooks:
    prek install

build: css
    cargo build --release
