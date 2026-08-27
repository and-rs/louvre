# Run the development server with Tailwind and Rust watchers.
run:
    #!/usr/bin/env bash
    if [[ -f .secrets/louvre-app-creds.json ]]; then
        export AWS_ACCESS_KEY_ID=$(awk -F'"' '/"AccessKeyId"/{print $4; exit}' .secrets/louvre-app-creds.json)
        export AWS_SECRET_ACCESS_KEY=$(awk -F'"' '/"SecretAccessKey"/{print $4; exit}' .secrets/louvre-app-creds.json)
    fi
    tailwindcss -i louvre-site/src/static/css/input.css -o louvre-site/src/static/css/site.css --silent
    tailwindcss -i louvre-site/src/static/css/input.css -o louvre-site/src/static/css/site.css --watch --silent &
    tailwind=$!
    trap 'kill "$tailwind" 2>/dev/null' EXIT INT TERM
    cargo watch -d 0.2 -w louvre-site/src -i louvre-site/src/static/css/site.css \
        -s "rustywind --write --output-css-file louvre-site/src/static/css/site.css louvre-site/src/templates && cargo run -p louvre-site --bin louvre --features dev"

# Format Rust, templates, styles, and JavaScript.
format:
    cargo fmt
    tailwindcss -i louvre-site/src/static/css/input.css -o louvre-site/src/static/css/site.css
    rustywind --write --output-css-file louvre-site/src/static/css/site.css louvre-site/src/templates
    biome format --write louvre-site/src/static/js/dev.js louvre-site/src/static/js/site.js louvre-site/src/static/css/input.css

# Run formatting, asset, lint, and test checks.
check:
    cargo fmt --check
    tailwindcss -i louvre-site/src/static/css/input.css -o louvre-site/src/static/css/site.css
    just compress
    rustywind --check-formatted --output-css-file louvre-site/src/static/css/site.css louvre-site/src/templates
    biome check louvre-site/src/static/js/dev.js louvre-site/src/static/js/site.js
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --all-features

# Brotli-compress browser assets.
compress:
    #!/usr/bin/env bash
    set -euo pipefail
    for f in louvre-site/src/static/css/site.css louvre-site/src/static/js/site.js louvre-site/src/static/js/mu.min.js; do
        brotli -q 11 -f "$f" -o "$f.br"
        printf '%s: %s -> %s\n' "$f" "$(du -h "$f" | cut -f1)" "$(du -h "$f.br" | cut -f1)"
    done

# Install the pre-commit hooks.
hooks:
    prek install

# Generate a Phosphor icon component.
icon name:
    ./scripts/icon.sh {{name}}

# Check the active AWS identity.
check-auth:
    aws sts get-caller-identity

# Configure Railway with the S3 credentials.
railway-s3:
    ./scripts/configure-railway-s3.sh

# Create the Terraform state bucket and lock table.
infra-bootstrap:
    ./scripts/bootstrap-state.sh

# Initialize Terraform.
infra-init:
    terraform -chdir=infra init -reconfigure

# Preview Terraform changes.
infra-plan:
    terraform -chdir=infra plan

# Apply Terraform changes.
infra-deploy:
    terraform -chdir=infra apply

# Create the Louvre app IAM credentials.
infra-app-key:
    ./scripts/create-app-key.sh

# Destroy the Terraform-managed infrastructure.
infra-destroy:
    terraform -chdir=infra destroy

# Build production assets and the release binary.
build:
    tailwindcss -i louvre-site/src/static/css/input.css -o louvre-site/src/static/css/site.css
    rustywind --write --output-css-file louvre-site/src/static/css/site.css louvre-site/src/templates
    just compress
    cargo build -p louvre-site --bin louvre --release
