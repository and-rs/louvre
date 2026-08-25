terraform {
  required_providers { aws = { source = "hashicorp/aws", version = "~> 5.0" } }
  backend "s3" {
    bucket       = "louvre-terraform-state"
    key          = "artworks/terraform.tfstate"
    region       = "us-east-1"
    use_lockfile = true
    encrypt      = true
  }
}

provider "aws" { region = var.target_region }

variable "target_region" {
  type        = string
  description = "The AWS region for louvre storage resources"
}

output "artworks_bucket" {
  value = aws_s3_bucket.artworks.bucket
}
