#!/bin/sh
set -eu

nix_bin="$(command -v nix 2>/dev/null || true)"
if [ -z "$nix_bin" ]; then
  for candidate in /nix/var/nix/profiles/default/bin/nix /run/current-system/sw/bin/nix; do
    if [ -x "$candidate" ]; then
      nix_bin="$candidate"
      break
    fi
  done
fi
if [ -z "$nix_bin" ]; then
  echo "Playwright MCP requires Nix; run it from this project's Nix environment." >&2
  exit 1
fi

exec "$nix_bin" develop --command npx --no-install @playwright/mcp \
  --isolated --headless --caps=devtools,testing
