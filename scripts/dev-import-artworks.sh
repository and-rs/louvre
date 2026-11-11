#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# Deliberately fixed: never use DATABASE_URL as the import destination.
local_database_url="postgres://louvre@127.0.0.1:55432/louvre"
psql "$local_database_url" -X -v ON_ERROR_STOP=1 -c 'SELECT 1' >/dev/null

services="$(railway service list --json)"
database="$(jq -er '
  [.[] | select((.source.image // "") | test("postgres"; "i")) | .name]
  | if length == 1 then .[0]
    else error("expected exactly one PostgreSQL service in the linked Railway environment") end
' <<<"$services")"

mkdir -p .local
dump="$(mktemp .local/artworks.XXXXXX.sql)"
trap 'rm -f -- "$dump"' EXIT

# Run inside Railway's private network; the database URL never leaves the container.
# Only artwork data; no production sessions, credentials, schema, or writes.
railway ssh --service "$database" \
  'PGOPTIONS="-c default_transaction_read_only=on" pg_dump "$DATABASE_URL" --data-only --table=public.artworks --column-inserts --on-conflict-do-nothing --no-owner --no-privileges' >"$dump"
if [[ ! -s "$dump" ]]; then
  printf 'error: Railway returned an empty artwork export; local data was not changed\n' >&2
  exit 1
fi
DATABASE_URL="$local_database_url" cargo run -p louvre-cli -- migrate
psql "$local_database_url" -X -v ON_ERROR_STOP=1 --single-transaction --file="$dump"
printf 'Imported missing artwork records into local PostgreSQL. Existing local records were preserved.\n'
