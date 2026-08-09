# Rust Site Baseline

A small integration proof of concept for a server-rendered marketing site.

- Axum owns routes and HTTP status codes.
- Maud renders complete HTML documents.
- `pulldown-cmark` renders Markdown on the server.
- Tailwind CSS runs through its standalone CLI.
- µJS progressively enhances links by replacing `<main>`.
- Anime.js is initialized by a small imperative module after both document and µJS navigation.

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
current generated stylesheet. Rust, Markdown, Tailwind input, and local
browser-JavaScript changes still reload the page. `src/static` contains browser
assets. `direnv allow` activates the Nix shell automatically when entering this
directory.

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
