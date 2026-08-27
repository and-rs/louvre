#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [[ -f .secrets/louvre-app-creds.json ]]; then
  export AWS_ACCESS_KEY_ID="$(awk -F'"' '/"AccessKeyId"/{print $4; exit}' .secrets/louvre-app-creds.json)"
  export AWS_SECRET_ACCESS_KEY="$(awk -F'"' '/"SecretAccessKey"/{print $4; exit}' .secrets/louvre-app-creds.json)"
fi

tailwindcss -i louvre-site/src/static/css/input.css -o louvre-site/src/static/css/site.css --silent
tailwindcss -i louvre-site/src/static/css/input.css -o louvre-site/src/static/css/site.css --watch --silent &
tailwind_pid=$!
trap 'kill "$tailwind_pid" 2>/dev/null || true' EXIT INT TERM

cargo watch -d 0.2 -w louvre-site/src -i louvre-site/src/static/css/site.css \
  -s "rustywind --write --output-css-file louvre-site/src/static/css/site.css louvre-site/src/templates && cargo run -p louvre-site --bin louvre --features dev"
