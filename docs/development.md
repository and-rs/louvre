# Local development

Enter `nix develop` and run `just run` (http://localhost:3000). Use
`just local-admin` for Studio access. On first run, migrate from another shell
and restart the app:

```sh
DATABASE_URL=postgres://louvre@127.0.0.1:55432/louvre just migrate
```

Use local PostgreSQL, not a tunnel to production.

**Import artwork:** Run `railway login` and `railway link` for the intended
environment, then `just dev-import-artworks` while `just run` is active. It reads
only artwork over Railway SSH, inserts missing records, and never overwrites
local edits or writes to production.

**Images:** Dev writes and deletes only in `.local/uploads` (`DEV_UPLOADS_DIR`).
Missing local images are read from S3. Run `just aws-login` for the default
profile or set `AWS_PROFILE` to a read-only profile.
