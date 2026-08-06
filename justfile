run:
    @cargo watch -w src -w content -s 'tailwindcss -i src/static/css/input.css -o src/static/css/site.css && rustywind --write --output-css-file src/static/css/site.css src/templates && cargo run --features dev'

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
