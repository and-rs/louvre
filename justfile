# Run the development server with Tailwind and Rust watchers.
run:
    ./scripts/run.sh

# Apply pending database migrations to DATABASE_URL.
migrate:
    cargo run -p louvre-site --bin louvre -- migrate

# Generate a stable Argon2 hash for STUDIO_PASSWORD_HASH.
studio-password:
    cargo run -p louvre-auth --features hash-cli --bin studio-password-hash

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

# Create an app access key and configure Railway to use the S3 bucket.
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

# Destroy the Terraform-managed infrastructure.
infra-destroy:
    terraform -chdir=infra destroy
