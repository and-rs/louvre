#!/usr/bin/env bash
set -euo pipefail

credentials_file="${1:-.secrets/louvre-app-creds.json}"

if [[ ! -f "$credentials_file" ]]; then
  echo "error: missing $credentials_file" >&2
  exit 1
fi

if ! railway status >/dev/null 2>&1; then
  echo "error: run 'railway login' and 'railway link' before configuring service variables" >&2
  exit 1
fi

credential_field() {
  awk -F '"' -v key="$1" '$2 == key { print $4; exit }' "$credentials_file"
}

access_key_id="$(credential_field AccessKeyId)"
secret_access_key="$(credential_field SecretAccessKey)"

if [[ -z "$access_key_id" || -z "$secret_access_key" ]]; then
  echo "error: $credentials_file does not contain an AWS access key" >&2
  exit 1
fi

railway variable set \
  AWS_REGION=us-east-1 \
  S3_BUCKET=louvre-artworks \
  AWS_EC2_METADATA_DISABLED=true \
  --skip-deploys
printf '%s' "$access_key_id" | railway variable set AWS_ACCESS_KEY_ID --stdin --skip-deploys
printf '%s' "$secret_access_key" | railway variable set AWS_SECRET_ACCESS_KEY --stdin

echo "configured Railway S3 variables and triggered a deployment"
