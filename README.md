# Louvre

Louvre is an artwork site and publishing platform. Its goal is to connect the full workflow: prepare artwork photos, manage source and web-ready assets in S3, publish a browsable collection, share work to Instagram, and sell artwork through the site.

The app is an early foundation today: it serves artwork images from S3 and has a minimal PostgreSQL connectivity endpoint. The collection, Instagram workflow, and purchasing experience are not yet built.

## Stack

- Rust with Axum and Tokio
- Server-rendered HTML with Maud
- Tailwind CSS
- AWS S3 for artwork assets
- PostgreSQL on Railway for application data
- Railway for deployment

## Development

```sh
nix develop
just run
```

`just run` starts local PostgreSQL 17 when `DATABASE_URL` is unset, keeps its data under `.local/postgres`, and stops it when the dev server exits. Set `DATABASE_URL` to use another database. Use PostgreSQL 17 for the Railway service as well.

Run the project checks with `just check`. Use `just` to see the available commands.

Create a PostgreSQL service in Railway and set the app service's `DATABASE_URL` variable to the PostgreSQL service's `DATABASE_URL` reference. The `/db/hello` endpoint runs a test query; without `DATABASE_URL`, it returns `503` while the rest of the app can still start.
