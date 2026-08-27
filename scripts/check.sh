#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo fmt --check
tailwindcss -i louvre-site/src/static/css/input.css -o louvre-site/src/static/css/site.css
"$repo_root/scripts/compress.sh"
rustywind --check-formatted --output-css-file louvre-site/src/static/css/site.css louvre-site/src/templates
biome check louvre-site/src/static/js/dev.js louvre-site/src/static/js/site.js
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
