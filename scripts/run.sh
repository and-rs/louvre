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

local_pgdata="$repo_root/.local/postgres"
local_pg_started=0
stop_local_postgres() {
  if (( local_pg_started )); then
    pg_ctl -D "$local_pgdata" -m fast -w stop >/dev/null || true
  fi
}
trap stop_local_postgres EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

if [[ -z "${DATABASE_URL:-}" ]]; then
  for command in initdb pg_ctl psql createdb; do
    if ! command -v "$command" >/dev/null 2>&1; then
      printf 'error: %s is unavailable; run this inside `nix develop`\n' "$command" >&2
      exit 1
    fi
  done

  if [[ ! -s "$local_pgdata/PG_VERSION" ]]; then
    mkdir -p "$local_pgdata"
    initdb -D "$local_pgdata" --username=louvre --auth-local=trust --auth-host=trust --no-instructions
  fi

  if ! pg_ctl -D "$local_pgdata" status >/dev/null 2>&1; then
    pg_ctl -D "$local_pgdata" \
      -o "-h 127.0.0.1 -p 55432 -k /tmp" \
      -l "$local_pgdata/server.log" \
      -w start
    local_pg_started=1
  fi

  database_exists="$(psql -h 127.0.0.1 -p 55432 -U louvre -d postgres -tAc \
    "SELECT 1 FROM pg_database WHERE datname = 'louvre'")"
  if [[ "$database_exists" != "1" ]]; then
    createdb -h 127.0.0.1 -p 55432 -U louvre louvre
  fi

  export DATABASE_URL="postgres://louvre@127.0.0.1:55432/louvre"
fi

cargo run -p louvre-site --bin louvre -- migrate

# Regenerate styles before restarting Axum; ignore generated CSS to avoid a watch loop.
cargo watch -d 0.2 \
  -w Cargo.toml -w Cargo.lock \
  -w louvre-auth/Cargo.toml -w louvre-auth/src \
  -w louvre-site/Cargo.toml -w louvre-site/src \
  -w louvre-storage/Cargo.toml -w louvre-storage/src \
  -i "$css_output" \
  -s "tailwindcss -i $css_input -o $css_output --silent && \
  rustywind --write --output-css-file $css_output $templates && \
  cargo run -p louvre-site --bin louvre --features dev"
