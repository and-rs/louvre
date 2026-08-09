# Agent Guidelines

## SVG Styling

- Use Tailwind utilities for all SVG paint and width: `fill-none`, `fill-*`, `stroke-*`, and `[stroke-width:*]`.
- Do not pair those utilities with `fill`, `stroke`, or `stroke-width` attributes, which can override Tailwind's layered utilities.
- Use semantic shadcn tokens such as `stroke-chart-1` and `fill-chart-2`; never use literal colors or CSS variables in markup.

## Template Styling

- Use semantic shadcn utilities (`bg-card`, `text-muted-foreground`, `border-border`) instead of literal colors in templates.
- Place reusable Maud components in `src/templates/components/` and re-export them through `src/templates/components/mod.rs`.
- Use the shared `max-w-5xl px-4` shell for navigation, main content, and footer; page-specific content may use a narrower nested max width.
