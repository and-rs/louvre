# Louvre

A small server-rendered site baseline.

- Axum owns routes and HTTP status codes.
- Maud renders complete HTML documents.
- Tailwind CSS runs through its standalone CLI.
- µJS progressively enhances links by replacing `<main>`.

## Development

Enter the reproducible development shell:

```bash
nix develop
```

Then start the development watchers:

```bash
just run
```

Open <http://127.0.0.1:3000>. Tailwind watches its sources independently, and
the server watcher ignores generated CSS so CSS rebuilds do not restart Rust.
Before each Rust restart, `rustywind` sorts Maud utility classes using the
current generated stylesheet. Rust and Tailwind input changes still reload the
page. `src/static` contains browser assets. `direnv allow` activates the Nix
shell automatically when entering this directory.

## Quality

```bash
just check
just hooks
```

`just hooks` installs the Prek pre-commit hook. Use `prek run --all-files` to
run the same pipeline across the repository.

## Build

```bash
just build
```

This writes minified CSS and compiles a release binary without development live
reload middleware.

## Deployment

Railway deploys automatically when the connected GitHub branch receives a
commit. `railway.json` selects the included multi-stage `Dockerfile`, which
generates minified Tailwind CSS and builds the release binary. The server binds
to Railway's injected `PORT` and exposes `/health` for zero-downtime deploys.

## Infrastructure

AWS infrastructure is defined in `infra/`. After configuring a default AWS CLI
profile, provision a fresh stack with:

```bash
just infra-bootstrap
just infra-init
just infra-plan
just infra-deploy
just infra-app-key
```

`infra-app-key` writes the restricted `louvre-app` credentials to the ignored
`.secrets/louvre-app-creds.json` file. Link the intended Railway service, then
configure its S3 variables without exposing the secret in shell history:

```bash
railway login
railway link
just railway-s3
```
