# Agent Guidelines

## Docs

- Do not document each justfile command in the README, just mention its existance.
- Do not churn README changes. Only edit README when the user requests.

## SVG Styling

- Use Tailwind utilities for all SVG paint and width: `fill-none`, `fill-*`, `stroke-*`, and `[stroke-width:*]`.
- Do not pair those utilities with `fill`, `stroke`, or `stroke-width` attributes, which can override Tailwind's layered utilities.
- Use semantic shadcn tokens such as `stroke-chart-1` and `fill-chart-2`; never use literal colors or CSS variables in markup.

## Template Styling

- **IMPERATIVE: Preserve reusable components, icons, their variants, and theme scaffolding (semantic tokens, light/dark declarations, and reusable CSS classes) even when currently unused.** They are intentional shared UI infrastructure, not dead code. Do not remove them or their supporting exports/dead-code allowances during cleanup unless the user explicitly requests their removal.
- Use semantic shadcn utilities (`bg-card`, `text-muted-foreground`, `border-border`) instead of literal colors in templates.
- Place reusable Maud components in `louvre-site/src/templates/components/` and re-export them through `louvre-site/src/templates/components/mod.rs`.
- Use the shared `max-w-4xl px-4` shell for navigation, main content, and footer; page-specific content may use a narrower nested max width.

## Navigation Links

- Always vendor JavaScript libraries locally; never load them from a CDN. Pin htmx to version 4.0.1 or newer, never below 4.0.1.
- htmx boosts same-origin navigation links and swaps `<main>` from full-page HTML responses. The pinned library is vendored in `louvre-site/src/static/js/`; use explicit `:inherited` attributes for shared navigation settings.
- Link to HTML pages only. Raw assets (images, ZIPs, downloads) must carry `hx-boost="false"`, or open via `target="_blank"` / `download`.
- Keep Studio forms opted out with `hx-boost="false"` unless explicitly implementing partial submissions. Keep htmx's server-refetched history; do not add a history-cache extension that could snapshot private Studio content.
- Images are embedded with `<img src=...>`, never as click targets of bare asset URLs.

## UI Verification

- For user-facing workflows, run `just e2e`; unit and route checks alone are not end-to-end verification.
