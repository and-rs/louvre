# Louvre

A small server-rendered artwork site built with Rust.

## Stack

- Axum and Tokio
- Maud templates
- Tailwind CSS
- AWS S3 artwork storage
- Railway deployment

## Workspace

- `louvre-site/` - web application and `louvre` binary
- `louvre-tw-merge/` - Tailwind class merger
- `infra/` - Terraform resources

## Commands

Run `just` or `just --list` for the documented command reference.

## Development

```sh
nix develop
just run
```

`just run` watches `louvre-site/src`. For each source change it regenerates
Tailwind CSS, orders template classes with Rustywind, and restarts Axum. The
development server injects a live-reload client, so the browser reloads after
the restart. Generated `site.css` is deliberately ignored by the watcher to
avoid a loop.

Run the complete local quality gate with:

```sh
just check
```

## Customization

This project is evolving into a public-site template. The current
customization points are:

- Site pages and route handlers: `louvre-site/src/routes.rs`
- Page templates: `louvre-site/src/templates/`
- Shared components: `louvre-site/src/templates/components/`
- Tailwind theme and local fonts: `louvre-site/src/static/css/input.css`
- Browser behavior: `louvre-site/src/static/js/site.js`

To add a page, create a template module, re-export it from
`louvre-site/src/templates/mod.rs`, add its handler to `routes.rs`, and add the
route in `main.rs`.

To add a shared component, place it in `templates/components/` and re-export it
through `templates/components/mod.rs`. Use semantic Tailwind tokens and the
shared shell described in `AGENTS.md`.

To add a Phosphor icon, run:

```sh
just icon <phosphor-name>
```

The command creates a Maud component in `templates/components/icons/` and
updates its exports. Review the generated icon against the SVG rules in
`AGENTS.md`.

Static files live under `louvre-site/src/static/`. Add browser assets to the
`ASSETS` list in `louvre-site/build.rs` when templates need a generated URL;
production builds attach a content version to those URLs. Same-origin links to
raw assets must use `data-mu="false"`, `target="_blank"`, or `download` so
µJS does not replace `<main>` with a non-HTML response.

## Production

The Docker build generates minified Tailwind CSS and a Brotli variant before
building the release binary. Production HTML references content-versioned CSS
and JavaScript URLs; `ServeDir` serves Brotli assets when the browser supports
them.

Railway uses `Dockerfile` and checks `/health`, as configured in
`railway.json`. The current artwork feature needs these service variables:

- `S3_BUCKET` (defaults to `louvre-artworks` if unset)
- `AWS_REGION`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`

Terraform manages the artwork bucket and its IAM user. After authenticating
with AWS and Railway, use `just infra-init`, `just infra-plan`,
`just infra-deploy`, `just infra-app-key`, and `just railway-s3` as needed.

## License

See `louvre-tw-merge/LICENSE` for the vendored merger's license.
