---
title: Faster by Default
description: A small article proving that Rust can own Markdown rendering without client hydration.
published_at: 2026-08-06
---

# Start with a document

The first response should be useful HTML. Navigation enhancement is optional,
not a prerequisite for content or links to work.

## The baseline

- Axum owns HTTP routes and status codes.
- Maud owns the document shape.
- Rust renders Markdown before it reaches the browser.
- Small JavaScript modules own browser-only behavior.

```rust
let app = Router::new().route("/", get(home));
```

That boundary is easy to inspect in browser developer tools and easy to test
with ordinary HTTP requests.
