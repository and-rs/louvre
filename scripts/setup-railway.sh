#!/usr/bin/env bash
set -euo pipefail

app_service="${RAILWAY_APP_SERVICE:-louvre}"

if ! railway status >/dev/null 2>&1; then
  echo "error: run 'railway login' and 'railway link' before setting up Railway" >&2
  exit 1
fi

services="$(railway service list --json)"
if ! jq -e --arg name "$app_service" 'any(.[]; .name == $name)' <<<"$services" >/dev/null; then
  echo "error: Railway app service '$app_service' does not exist in the linked environment" >&2
  exit 1
fi

postgres_services() {
  railway service list --json | jq -r '[.[] | select((.source.image // "") | test("postgres"; "i")) | .name] | .[]'
}

databases=()
while IFS= read -r database; do
  databases+=("$database")
done < <(postgres_services)
if ((${#databases[@]} == 0)); then
  railway add --database postgres --json >/dev/null
  while IFS= read -r database; do
    databases+=("$database")
  done < <(postgres_services)
fi

if ((${#databases[@]} != 1)); then
  echo "error: expected exactly one PostgreSQL service in the linked environment; found ${#databases[@]}" >&2
  exit 1
fi

printf -v database_reference '${{%s.DATABASE_URL}}' "${databases[0]}"
variables="$(railway variable list --service "$app_service" --json)"
current_site_name="$(jq -r '.SITE_NAME // empty' <<<"$variables")"
default_site_name="${current_site_name:-Louvre}"
read -r -p "Site name [$default_site_name]: " site_name
site_name="${site_name:-$default_site_name}"

variables_to_set=()
if ! jq -e --arg expected "$database_reference" '.DATABASE_URL == $expected' <<<"$variables" >/dev/null; then
  variables_to_set+=("DATABASE_URL=$database_reference")
else
  echo "Railway PostgreSQL service '${databases[0]}' and app DATABASE_URL are already configured."
fi

if [[ "$site_name" != "$current_site_name" ]]; then
  variables_to_set+=("SITE_NAME=$site_name")
fi

if ((${#variables_to_set[@]} > 0)); then
  railway variable set "${variables_to_set[@]}" --service "$app_service"
fi
printf "Configured Railway app '%s' with PostgreSQL '%s' and site name '%s'.\n" "$app_service" "${databases[0]}" "$site_name"
