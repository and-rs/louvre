# Louvre

Louvre is an artwork site and publishing platform. Its goal is to connect the full workflow: prepare artwork photos, manage source and web-ready assets in S3, publish a browsable collection, share work to Instagram, and sell artwork through the site.

The app is an early foundation today: it serves artwork images from S3, but the collection, Instagram workflow, and purchasing experience are not yet built.

## Stack

- Rust with Axum and Tokio
- Server-rendered HTML with Maud
- Tailwind CSS
- AWS S3 for artwork assets
- Railway for deployment

## Development

```sh
nix develop
just run
```

Run the project checks with `just check`. Use `just` to see the available commands.
