run:
    #!/usr/bin/env bash
    if [[ -f .secrets/sanarte-app-creds.json ]]; then
        export AWS_ACCESS_KEY_ID=$(awk -F'"' '/"AccessKeyId"/{print $4; exit}' .secrets/sanarte-app-creds.json)
        export AWS_SECRET_ACCESS_KEY=$(awk -F'"' '/"SecretAccessKey"/{print $4; exit}' .secrets/sanarte-app-creds.json)
    fi
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

check-auth:
    aws sts get-caller-identity

infra-bootstrap:
    ./scripts/bootstrap-state.sh

infra-init:
    terraform -chdir=infra init

infra-plan:
    terraform -chdir=infra plan

infra-deploy:
    terraform -chdir=infra apply

infra-destroy:
    terraform -chdir=infra destroy

build:
    tailwindcss -i src/static/css/input.css -o src/static/css/site.css
    rustywind --write --output-css-file src/static/css/site.css src/templates
    just compress
    cargo build --release
