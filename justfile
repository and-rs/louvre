# Run the development server with Tailwind and Rust watchers.
run:
    ./scripts/run.sh

# Format Rust, templates, styles, and JavaScript.
format:
    ./scripts/format.sh

# Run formatting, asset, lint, and test checks.
check:
    ./scripts/check.sh

# Brotli-compress browser assets.
compress:
    ./scripts/compress.sh

# Install the pre-commit hooks.
hooks:
    prek install

# Generate a Phosphor icon component.
icon name:
    ./scripts/icon.sh {{name}}

# Check the active AWS identity.
check-auth:
    aws sts get-caller-identity

# Configure Railway with the S3 credentials.
railway-s3:
    ./scripts/configure-railway-s3.sh

# Create the Terraform state bucket and lock table.
infra-bootstrap:
    ./scripts/bootstrap-state.sh

# Initialize Terraform.
infra-init:
    terraform -chdir=infra init -reconfigure

# Preview Terraform changes.
infra-plan:
    terraform -chdir=infra plan

# Apply Terraform changes.
infra-deploy:
    terraform -chdir=infra apply

# Create the Louvre app IAM credentials.
infra-app-key:
    ./scripts/create-app-key.sh

# Destroy the Terraform-managed infrastructure.
infra-destroy:
    terraform -chdir=infra destroy
