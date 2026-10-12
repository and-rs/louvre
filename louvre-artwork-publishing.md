# Artwork Publishing Vertical Slice

## Goal

Complete one useful product loop:

**Sign in → create a draft artwork → publish it → find and view it on the public site.**

## Current baseline

- `artworks` already stores a UUID, unique slug, title, description, collection, medium, collection number, status, timestamps, and ordered image metadata in JSONB.
- Status is constrained to `draft`, `published`, or `archived`; `published_at` already exists.
- `/studio` is authenticated but only shows a placeholder.
- `/` is empty. `/artwork/{id}` currently lists files from S3 without loading the artwork record.
- S3 storage supports listing and reading objects, not uploading or transforming them.

## Implementation

### 1. Artwork persistence and rules

- Reuse the existing table; expect no migration unless implementation reveals a required field that the schema cannot represent.
- Add a small typed artwork query/model module in `louvre-storage` for listing, creating, editing, publishing, and archiving records. Avoid a generic repository abstraction.
- Validate required title and slug, slug format, unique-slug conflicts, and image metadata. Proposed initial rule: drafts may be incomplete; publishing requires at least one selected image.
- Update `updated_at` on edits and set `published_at` on first publication. Keep the public slug stable after publication to avoid breaking links.

### 2. Authenticated Studio workflow

- Turn `/studio` into an artwork dashboard showing title, slug, and draft/published/archived status, with a “New artwork” action.
- Add create and edit forms for title, slug, description, collection, medium, collection number, and image ordering/alt text.
- For this slice, let the owner select existing image objects under `artworks/{slug}/` and store references in the documented `images` JSONB shape. Do not add upload or image-processing UI yet.
- Add save-draft, publish, and archive actions. Protect every Studio management route with the existing login middleware; keep login/logout behavior unchanged.
- Return validation failures to the form with entered values preserved and clear feedback, including duplicate slugs.

### 3. Public collection and artwork pages

- Make `/` query only published records and render a collection linking to each detail page. Order by `collection_number` ascending with nulls last, then use a stable secondary order. Show a useful empty state when nothing is published.
- Make `/artwork/{slug}` load the published record from PostgreSQL and render its metadata and ordered images from the record, rather than discovering artwork solely by listing S3.
- Keep the existing server-side image proxy, but resolve image keys from the published record and only serve keys referenced by that artwork. Drafts and archived records—and their image URLs—must return 404 publicly.
- Preserve the open, editorial visual direction; avoid bringing back generic centered cards.

### 4. Verification

- Add tests for draft persistence and edits, publishing timestamps/status, archive visibility, and unique-slug failures against an isolated PostgreSQL test database.
- Add route tests for signed-out access to management actions, published-only public pages, and hidden draft/archived image URLs.
- Manually exercise: sign in → create draft → save → publish → find on Home → open detail → archive → confirm it disappears.
- Smoke-test image loading with a representative S3 object, then run `nix develop -c just check`.

## Scope and assumptions

- The first slice references images already in S3; a browser upload pipeline, resizing/derivatives, payments, Instagram, analytics, and multi-user account management are out of scope.
- The current environment-based single Studio account remains the author identity.
- Publishing requires at least one selected image. If publishing text-only artworks is desired, relax this rule before implementation.
- The end-to-end smoke test needs a configured database and at least one suitable S3 image. Local S3 emulation is a separate task.
