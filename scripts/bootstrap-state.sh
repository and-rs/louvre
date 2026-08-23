#!/usr/bin/env bash
set -euo pipefail

region="${AWS_REGION:-us-east-1}"
bucket="sanarte-terraform-state"

if aws s3api head-bucket --bucket "$bucket" 2>/dev/null; then
  echo "state bucket '$bucket' already exists"
  exit 0
fi

if [[ "$region" == "us-east-1" ]]; then
  aws s3api create-bucket --bucket "$bucket" --region "$region"
else
  aws s3api create-bucket \
    --bucket "$bucket" \
    --region "$region" \
    --create-bucket-configuration "LocationConstraint=$region"
fi

aws s3api put-bucket-versioning \
  --bucket "$bucket" \
  --versioning-configuration "Status=Enabled"

echo "created $bucket in $region"
