#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
mode="${1:-test}"
if [[ "$mode" != "test" && "$mode" != "serve" ]]; then
  printf 'usage: %s [test|serve]\n' "$0" >&2
  exit 2
fi

tmp_root="${TMPDIR:-${RUNNER_TEMP:-$HOME/.cache}}"
mkdir -p "$tmp_root"
tmp_dir="$(mktemp -d "$tmp_root/louvre-e2e.XXXXXX")"
pgdata="$tmp_dir/postgres"
pg_port="$(node -e 'const s=require("node:net").createServer();s.listen(0,"127.0.0.1",()=>{console.log(s.address().port);s.close()})')"
s3_port="$(node -e 'const s=require("node:net").createServer();s.listen(0,"127.0.0.1",()=>{console.log(s.address().port);s.close()})')"

app_pid=""
s3_pid=""
cleanup() {
  if [[ -n "$app_pid" ]]; then
    kill -KILL "$app_pid" >/dev/null 2>&1 || true
    wait "$app_pid" >/dev/null 2>&1 || true
  fi
  if [[ -n "$s3_pid" ]]; then
    kill "$s3_pid" >/dev/null 2>&1 || true
    wait "$s3_pid" >/dev/null 2>&1 || true
  fi
  pg_ctl -D "$pgdata" -m fast -w stop >/dev/null 2>&1 || true
  rm -rf -- "$tmp_dir"
}
trap cleanup EXIT

initdb -D "$pgdata" --username=louvre --auth-local=trust --auth-host=trust --no-instructions >/dev/null
pg_ctl -D "$pgdata" \
  -o "-h 127.0.0.1 -p $pg_port -k $tmp_dir" \
  -l "$tmp_dir/postgres.log" -w start >/dev/null
createdb -h 127.0.0.1 -p "$pg_port" -U louvre louvre_e2e

export DATABASE_URL="postgres://louvre@127.0.0.1:$pg_port/louvre_e2e"
export PORT=3300
export S3_BUCKET=louvre-e2e
export DEV_UPLOADS_DIR="$tmp_dir/uploads"
export S3_MOCK_PORT="$s3_port"
export S3_ENDPOINT_URL="http://127.0.0.1:$s3_port"
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=e2e
export AWS_SECRET_ACCESS_KEY=e2e
export AWS_EC2_METADATA_DISABLED=true
unset AWS_PROFILE AWS_DEFAULT_PROFILE || true
export STUDIO_USERNAME=e2e-owner
export STUDIO_PASSWORD_HASH='$argon2id$v=19$m=19456,t=2,p=1$cALHoRXElPrrfe9XXlOiuw$G8hjD5xiViSm5543nacL3X1nf/64VcAlytxG2P3xkqc'

tailwindcss -i louvre-site/src/static/css/input.css -o louvre-site/src/static/css/site.css --silent
cargo run -p louvre-cli -- migrate

if [[ "$mode" == "serve" ]]; then
  node e2e/s3-mock.mjs &
  s3_pid=$!
  cargo build -p louvre-site --bin louvre-server --features dev
  target/debug/louvre-server &
  app_pid=$!
  printf 'E2E site: http://127.0.0.1:%s\n' "$PORT"
  printf 'Studio login: %s / e2e-password\n' "$STUDIO_USERNAME"
  printf 'Test database, local uploads, and read-only S3 fixture are isolated; press Ctrl-C to stop.\n'
  wait "$app_pid"
else
  npm run e2e
fi
