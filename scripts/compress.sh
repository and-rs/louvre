#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

for file in louvre-site/src/static/css/site.css louvre-site/src/static/js/site.js louvre-site/src/static/js/mu.min.js; do
  brotli -q 11 -f "$file" -o "$file.br"
  printf '%s: %s -> %s\n' "$file" "$(du -h "$file" | cut -f1)" "$(du -h "$file.br" | cut -f1)"
done
