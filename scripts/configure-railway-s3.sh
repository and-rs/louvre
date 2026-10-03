#!/usr/bin/env bash
set -euo pipefail

if ! railway status >/dev/null 2>&1; then
  echo "error: run 'railway login' and 'railway link' before configuring service variables" >&2
  exit 1
fi

key_count="$(aws iam list-access-keys \
  --user-name louvre-app \
  --query 'length(AccessKeyMetadata)' \
  --output text)"
if (( key_count >= 2 )); then
  echo "error: louvre-app already has two access keys; remove an old key before rotating" >&2
  exit 1
fi

credentials="$(aws iam create-access-key \
  --user-name louvre-app \
  --query 'AccessKey.[AccessKeyId,SecretAccessKey]' \
  --output text)"
IFS=$'\t' read -r access_key_id secret_access_key <<< "$credentials"
unset credentials

if [[ -z "${access_key_id:-}" || -z "${secret_access_key:-}" ]]; then
  echo "error: AWS did not return an access key pair" >&2
  exit 1
fi

railway variable set \
  "AWS_REGION=${AWS_REGION:-us-east-1}" \
  S3_BUCKET=louvre-artworks \
  AWS_EC2_METADATA_DISABLED=true \
  --skip-deploys
printf '%s' "$access_key_id" | railway variable set AWS_ACCESS_KEY_ID --stdin --skip-deploys
printf '%s' "$secret_access_key" | railway variable set AWS_SECRET_ACCESS_KEY --stdin
unset access_key_id secret_access_key

echo "configured Railway S3 credentials and triggered a deployment; retire any previous app key after verifying it"
