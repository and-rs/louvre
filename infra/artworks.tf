resource "aws_s3_bucket" "artworks" {
  bucket = "louvre-artworks"
}

resource "aws_s3_bucket_public_access_block" "artworks" {
  bucket = aws_s3_bucket.artworks.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_ownership_controls" "artworks" {
  bucket = aws_s3_bucket.artworks.id

  rule {
    object_ownership = "BucketOwnerEnforced"
  }
}
