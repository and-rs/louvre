# Site Roadmap

## 1. Phosphor Icons

- [x] Use Phosphor Icons, not Lucide.
- [x] Store reusable icon Maud components in `louvre-site/src/templates/components/icons/` and re-export them through `louvre-site/src/templates/components/icons/mod.rs`.
- [x] Icons must follow the SVG rules in `AGENTS.md`: Tailwind-only paint and stroke-width utilities with semantic shadcn tokens.
- [x] Add a `just icon <name>` intake command backed by a small script.
- [x] The script should download a chosen Phosphor SVG weight, convert its SVG elements to Maud markup, remove conflicting paint attributes, and create the component file.
- [x] Start with the icons needed for the footer and external links: GitHub, LinkedIn, ArrowUpRight, Envelope, and ArrowRight.

## 2. Footer

- [x] Replace the current paragraph-only footer with a structured responsive layout.
- [x] Add external profiles and a contact CTA when the template receives a product identity.
- [x] Use reusable buttons for the CTA and reusable Phosphor icons for external destinations.
- [x] Keep the footer inside the existing `max-w-5xl` shell and use semantic shadcn utilities only.

## 3. Analytics

- Use self-hosted Umami on Railway with a separate Railway PostgreSQL service. Do not add Grafana.
- Configure Umami with the Railway Postgres `DATABASE_URL`, UTC timestamps, a durable admin password, and a public analytics domain or service URL.
- Add the tracker script only when `UMAMI_WEBSITE_ID` and `UMAMI_SCRIPT_URL` are configured, so local development has no analytics dependency.
- Track only meaningful events: home CTA clicks, outbound work links, and footer contact/profile links.
- Use Umami's dashboard, goals, funnels, and performance reporting before considering another observability system.

## 4. Metadata And Discovery

- Expand page metadata beyond title and description: canonical URL, Open Graph title/description/type/url/image, Twitter card metadata, and theme color.
- Refactor the page template input into a metadata struct so routes provide page-specific values consistently.
- Add `/robots.txt` and `/sitemap.xml`; generate sitemap entries from the known static routes.
- Add a social preview image once there is a stable brand asset.

## Delivery Order

1. Phosphor intake script and first reusable icons.
2. Footer rebuild.
3. Metadata, robots, and sitemap.
4. Railway Postgres plus Umami, then event tracking.
