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

## License

See `louvre-tw-merge/LICENSE` for the vendored merger's license.
