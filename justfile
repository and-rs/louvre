# Load optional local configuration from the ignored .env file.
set dotenv-load := true

# Run the development server with Tailwind and Rust watchers.
run:
  ./scripts/run.sh

# Apply pending database migrations to DATABASE_URL.
migrate:
  @cargo run -p louvre-cli -- migrate

# Prompt for studio admin credentials and configure the linked Railway app.
railway-admin:
  ./scripts/set-railway-admin.sh

# Prompt for studio admin credentials and save them to the ignored local .env.
local-admin:
  @cargo run -p louvre-cli -- admin local

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

# Ensure Railway has PostgreSQL and the app references its DATABASE_URL.
railway-setup:
  ./scripts/setup-railway.sh

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
