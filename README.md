# Louvre

Louvre is an artwork site and publishing platform. Its goal is to connect the full workflow: prepare artwork photos, manage source and web-ready assets in S3, publish a browsable collection, share work to Instagram, and sell artwork through the site.

The app is an early foundation today: it serves artwork images from S3 and has an initial PostgreSQL artwork schema. The collection, Instagram workflow, and purchasing experience are not yet built.

## Stack

- Rust with Axum and Tokio
- Server-rendered HTML with Maud
- Tailwind CSS
- AWS S3 for artwork assets
- SQLx for asynchronous PostgreSQL access and migrations
- PostgreSQL on Railway for application data
- Railway for deployment

## Development

```sh
nix develop
just run
```

`just run` starts local PostgreSQL 17 when `DATABASE_URL` is unset, keeps its data under `.local/postgres`, applies pending migrations, and stops it when the dev server exits. The app requires `DATABASE_URL` and PostgreSQL 17. Run `just migrate` to apply migrations explicitly.

The private studio is available at `/studio` when `STUDIO_USERNAME` and `STUDIO_PASSWORD_HASH` are set. Use `just studio-password` to generate the Argon2 hash without echoing the password. Sessions are stored in PostgreSQL, shared across backend instances, and expire after eight hours of inactivity. Production cookies are secure and HTTP-only; production access requires HTTPS.

Run the project checks with `just check`. Use `just` to see the available commands.

Create a PostgreSQL 17 service in Railway and set the app service's `DATABASE_URL` variable to the PostgreSQL service's `DATABASE_URL` reference. Set the studio credentials in the app service's environment. Railway runs `louvre migrate` as a pre-deploy command. Database changes live in `louvre-storage/migrations`; `louvre-auth` manages the shared session store.

The initial `artworks` schema stores artwork metadata in one row. `collection_number` is an optional, manually assigned integer. The `images` JSONB array keeps each image's alt text and order together with its source and web-ready S3 object keys.
