#!/usr/bin/env bash
set -euo pipefail

credentials_file=".secrets/louvre-app-creds.json"

if [[ -e "$credentials_file" ]]; then
  echo "error: $credentials_file already exists; refuse to create another IAM access key" >&2
  exit 1
fi

mkdir -p .secrets
chmod 700 .secrets
aws iam create-access-key --user-name louvre-app >"$credentials_file"
chmod 600 "$credentials_file"

echo "created Louvre app credentials in $credentials_file"
