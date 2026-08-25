resource "aws_iam_user" "app" {
  name = "louvre-app"
}

resource "aws_iam_user_policy" "app" {
  name = "louvre-app-artworks"
  user = aws_iam_user.app.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "ListArtworksBucket"
        Effect = "Allow"
        Action = [
          "s3:ListBucket",
          "s3:GetBucketLocation",
        ]
        Resource = [aws_s3_bucket.artworks.arn]
      },
      {
        Sid    = "ManageArtworkObjects"
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject",
        ]
        Resource = ["${aws_s3_bucket.artworks.arn}/*"]
      },
    ]
  })
}
