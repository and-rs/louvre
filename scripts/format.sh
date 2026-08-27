#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo fmt
tailwindcss -i louvre-site/src/static/css/input.css -o louvre-site/src/static/css/site.css
rustywind --write --output-css-file louvre-site/src/static/css/site.css louvre-site/src/templates
biome format --write louvre-site/src/static/js/dev.js louvre-site/src/static/js/site.js louvre-site/src/static/css/input.css
