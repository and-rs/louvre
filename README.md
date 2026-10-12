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

Enter the development environment with Nix. The repository includes a justfile for common development and check tasks.

Local settings can be supplied through the ignored `.env` file; see `.env.example`. When `DATABASE_URL` is unset, the local development setup uses PostgreSQL 17 with data stored under `.local/postgres`. The application requires PostgreSQL, and pending migrations must be applied before startup.

Studio sign-in is configured with `STUDIO_USERNAME` and `STUDIO_PASSWORD_HASH` environment variables. Use the ignored `.env` file locally and the Railway app's environment settings in production; do not store the plaintext password there. Sessions are stored in PostgreSQL, shared across backend instances, and expire after eight hours of inactivity. Production cookies are secure and HTTP-only; production access requires HTTPS.

Railway runs database migrations before deploying the app. Configure the app's `DATABASE_URL` as a reference to the Railway PostgreSQL service. Database schema changes live in `louvre-storage/migrations`; `louvre-auth` manages the shared session store.

The initial `artworks` schema stores artwork metadata in one row. `collection_number` is an optional, manually assigned integer. The `images` JSONB array keeps each image's alt text and order together with its source and web-ready S3 object keys.
