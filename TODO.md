# Site Roadmap

## 1. Article Code Blocks

- Style Markdown `pre` and `code` output in the article prose container.
- Give fenced blocks a neutral shadcn surface: `bg-muted`, `border-border`, rounded corners, responsive padding, horizontal overflow, and readable monospace line height.
- Style inline code separately so it does not look like a full code block.
- Keep the first pass language-neutral. Add syntax highlighting only when articles need it.

## 2. Phosphor Icons

- Use Phosphor Icons, not Lucide.
- Store reusable icon Maud components in `src/templates/components/icons/` and re-export them through `src/templates/components/icons/mod.rs`.
- Icons must follow the SVG rules in `AGENTS.md`: Tailwind-only paint and stroke-width utilities with semantic shadcn tokens.
- Add a `just icon <name>` intake command backed by a small script.
- The script should download a chosen Phosphor SVG weight, convert its SVG elements to Maud markup, remove conflicting paint attributes, and create the component file.
- Start with the icons needed for the footer and external links: GitHub, LinkedIn, ArrowUpRight, Envelope, and ArrowRight.

## 3. Footer

- Replace the current paragraph-only footer with a structured responsive layout.
- Include brand and positioning copy, internal navigation, external profiles, and a contact CTA.
- Use reusable buttons for the CTA and reusable Phosphor icons for external destinations.
- Keep the footer inside the existing `max-w-5xl` shell and use semantic shadcn utilities only.

## 4. Analytics

- Use self-hosted Umami on Railway with a separate Railway PostgreSQL service. Do not add Grafana.
- Configure Umami with the Railway Postgres `DATABASE_URL`, UTC timestamps, a durable admin password, and a public analytics domain or service URL.
- Add the tracker script only when `UMAMI_WEBSITE_ID` and `UMAMI_SCRIPT_URL` are configured, so local development has no analytics dependency.
- Track only meaningful events: home CTA clicks, outbound work links, and footer contact/profile links.
- Use Umami's dashboard, goals, funnels, and performance reporting before considering another observability system.

## 5. Metadata And Discovery

- Expand page metadata beyond title and description: canonical URL, Open Graph title/description/type/url/image, Twitter card metadata, and theme color.
- Refactor the page template input into a metadata struct so routes provide page-specific values consistently.
- Add `/robots.txt` and `/sitemap.xml`; generate sitemap entries from the known static routes and article slugs.
- Add a social preview image once there is a stable brand asset.

## 6. Homepage Metrics

- Explain the performance panel as a live measurement of the visitor's current page load, not a claim about a fixed service-level result.
- Add a short interpretation line near the metric heading that distinguishes server responsiveness, rendering speed, and visual stability.
- Group or visually label the metrics by phase: response (`TTFB`), rendering (`DOM Ready`, `FP`, `FCP`, `LCP`), and stability (`CLS`).
- Add concise descriptions of what a good or poor value means, without presenting misleading universal thresholds.
- Preserve graceful `N/A` behavior for browsers that do not expose a metric.

## 7. Spa Animation Cleanup

- Reproduce navigation away from and back to the home page, including browser tab visibility changes, to confirm the residual trail behavior.
- When µJS emits `mu:before-render`, cancel all Anime animations and reset the Spa SVG elements before `<main>` is replaced.
- Reset trace opacity and drawable state, car opacity and transform, and any generated stroke-dash values left by Anime's drawable helper.
- Handle `visibilitychange` so a hidden tab does not resume with stale timeline state.
- Keep animation setup idempotent after `mu:after-render` and respect `prefers-reduced-motion`.

## Delivery Order

1. Article code-block styling.
2. Phosphor intake script and first reusable icons.
3. Footer rebuild.
4. Homepage metric explanation and Spa cleanup.
5. Metadata, robots, and sitemap.
6. Railway Postgres plus Umami, then event tracking.
