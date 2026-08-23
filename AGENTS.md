# Agent Guidelines

## SVG Styling

- Use Tailwind utilities for all SVG paint and width: `fill-none`, `fill-*`, `stroke-*`, and `[stroke-width:*]`.
- Do not pair those utilities with `fill`, `stroke`, or `stroke-width` attributes, which can override Tailwind's layered utilities.
- Use semantic shadcn tokens such as `stroke-chart-1` and `fill-chart-2`; never use literal colors or CSS variables in markup.

## Template Styling

- Use semantic shadcn utilities (`bg-card`, `text-muted-foreground`, `border-border`) instead of literal colors in templates.
- Place reusable Maud components in `src/templates/components/` and re-export them through `src/templates/components/mod.rs`.
- Use the shared `max-w-4xl px-2` shell for navigation, main content, and footer; page-specific content may use a narrower nested max width.

## Navigation Links

- µJS intercepts clicks on every same-origin anchor and swaps the response into `<main>`.
- Link to HTML pages only. Raw assets (images, ZIPs, downloads) must carry `data-mu="false"`, or open via `target="_blank"` / `download`.
- Images are embedded with `<img src=...>`, never as click targets of bare asset URLs.
