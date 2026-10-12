#!/usr/bin/env bash
set -euo pipefail

app_service="${RAILWAY_APP_SERVICE:-louvre}"

if ! railway status >/dev/null 2>&1; then
  echo "error: run 'railway login' and 'railway link' before configuring admin credentials" >&2
  exit 1
fi

if ! railway service list --json | jq -e --arg name "$app_service" 'any(.[]; .name == $name)' >/dev/null; then
  echo "error: Railway app service '$app_service' does not exist in the linked environment" >&2
  exit 1
fi

read -r -p "Studio username: " username
if [[ -z "${username//[[:space:]]/}" ]]; then
  echo "error: username cannot be empty" >&2
  exit 1
fi

password_hash="$(cargo run --quiet -p louvre-cli -- admin hash)"
if [[ -z "$password_hash" ]]; then
  echo "error: password hash was not generated" >&2
  exit 1
fi

railway variable set "STUDIO_USERNAME=$username" --service "$app_service" --skip-deploys
printf '%s' "$password_hash" | railway variable set STUDIO_PASSWORD_HASH --stdin --service "$app_service" --skip-deploys
unset password_hash

railway service redeploy --service "$app_service" --yes
printf "Configured studio credentials on Railway service '%s'.\n" "$app_service"
