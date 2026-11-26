# Local development

Enter `nix develop` and run `just run` (http://localhost:3000). On first run,
migrate from another shell:

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

## Publish artwork

```sh
just artwork new quiet-study "Quiet Study"
just artwork add-image quiet-study ./study.jpg "A charcoal study of a tree"
just artwork publish quiet-study
```

Use `just artwork --help` for editing and image ordering. Local publishes do
not sync to production, and local artwork commands must not use a production tunnel.

For production, use the intended app service’s Railway SSH target (copy its
SSH command from the dashboard). Run the same commands with
`louvre artwork --railway`. Stream a file from your laptop with OpenSSH `-T`
and use `-` in place of the remote filename:

```sh
ssh -T APP_SERVICE_DOMAIN@ssh.railway.com \
  "louvre artwork --railway add-image quiet-study - 'A charcoal study of a tree'" < study.jpg
```

Create the draft remotely first, then publish it remotely after inspecting it
with `show`. If a connection drops during upload, check `show` before retrying.
Remove the old `STUDIO_USERNAME` and `STUDIO_PASSWORD_HASH` deployment variables
after the cutover.
