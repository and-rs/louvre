# Development data and images

Run `nix develop`, `just local-admin`, `just aws-login`, then `just run`.
The app is at http://localhost:3000 unless `PORT` is set.

`just run` manages PostgreSQL at `127.0.0.1:55432`. On first use, keep it
running and apply migrations in another development shell:

```sh
DATABASE_URL=postgres://louvre@127.0.0.1:55432/louvre just migrate
```

Restart the app after initial migration so session cleanup starts successfully.
Dev builds reject non-loopback PostgreSQL hosts. Do not use a tunnel to a
production database as your development database.

## Existing production artwork

Log in with `railway login` and link the intended production project/environment
with `railway link`. With `just run` keeping local PostgreSQL running:

```sh
just dev-import-artworks
```

The command detects the single PostgreSQL service in the linked Railway
environment, using the same image-based detection as `just railway-setup`. It
aborts if there are zero or multiple PostgreSQL services. It runs `pg_dump`
inside the database container over Railway SSH; PostgreSQL stays private, and
no `PRODUCTION_DATABASE_URL` or public TCP proxy is needed. Your SSH key may
prompt for its passphrase.

This reads only `public.artworks` from production and inserts missing records
into the managed local database. Existing local records (including edits) win
on ID or slug conflicts. It does not copy sessions or change production data.
It is an initial import, not a refresh of existing records. Database migrations
must be compatible between production and this checkout. Failed or empty
exports abort before local migrations/import; temporary exports are deleted
automatically.

## Image storage

Dev builds read images from `.local/uploads` first, falling back to real S3
when a file is absent. Uploads and deletes affect only local files; deleting an
imported image only removes its local database reference. Dev never sends S3
PUT or DELETE requests. Existing `s3_key` fields also identify local files;
there is no schema change or S3 emulator needed for development.

Set `S3_BUCKET` if it differs from `louvre-artworks`. AWS uses the normal SDK
credential chain (`just aws-login` authenticates the default profile used by
the Nix shell). Dev no longer loads `.secrets/louvre-app-creds.json`. For
defense in depth, use an AWS profile with only `s3:GetObject` permission for
the artwork bucket, rather than production app write credentials. This code
change does not provision or change AWS credentials or IAM permissions.

`DEV_UPLOADS_DIR` overrides the local upload directory when isolation is needed
(for example, browser tests). Production builds continue to use S3 for reads,
uploads, and deletes. Do not copy local image references back into production.
