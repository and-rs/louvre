#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [[ -f .secrets/louvre-app-creds.json ]]; then
  AWS_ACCESS_KEY_ID="$(awk -F'"' '/"AccessKeyId"/{print $4; exit}' .secrets/louvre-app-creds.json)"
  AWS_SECRET_ACCESS_KEY="$(awk -F'"' '/"SecretAccessKey"/{print $4; exit}' .secrets/louvre-app-creds.json)"

  export AWS_ACCESS_KEY_ID
  export AWS_SECRET_ACCESS_KEY
fi

css_input="louvre-site/src/static/css/input.css"
css_output="louvre-site/src/static/css/site.css"
templates="louvre-site/src/templates"

tailwindcss -i "$css_input" -o "$css_output" --silent

# Regenerate styles before restarting Axum; ignore generated CSS to avoid a watch loop.
cargo watch -d 0.2 -w louvre-site/src -i "$css_output" \
  -s "tailwindcss -i $css_input -o $css_output --silent && \
  rustywind --write --output-css-file $css_output $templates && \
  cargo run -p louvre-site --bin louvre --features dev"
