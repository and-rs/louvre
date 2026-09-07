# Template Roadmap

## 1. Local-First Foundation

- [ ] Move site identity into one typed config: name, public URL, description,
      social image, footer links, and optional analytics settings.
- [ ] Make local development work without AWS credentials or an S3 bucket.
- [ ] Make artwork/S3 storage an optional example capability rather than a
      startup requirement.
- [ ] Add `.env.example` and document required versus optional environment
      variables.
- [ ] Validate required production configuration at startup.

## 2. Public-Site Baseline

- [ ] Refactor page inputs into a metadata struct.
- [ ] Add canonical, Open Graph, Twitter, and theme-color metadata.
- [ ] Add `/robots.txt` and `/sitemap.xml` from known static routes.
- [ ] Add a social preview image once branding is stable.
- [x] Keep the shared layout, theme toggle, footer, buttons, and Phosphor icon
      workflow as the default design system.

## 3. Developer Experience

- [x] Run Tailwind, Rustywind, and Axum through one development watch flow.
- [x] Reload the browser after source changes without CSS polling.
- [x] Document how to add a route, page, component, icon, and static asset.
- [x] Document the local development and production build flows.
- [ ] Add a concise "start a new site from this template" guide.

## 4. Deployment And Operations

- [x] Build content-versioned static URLs for production.
- [x] Serve Brotli-compressed static assets in production.
- [x] Add graceful shutdown.
- [ ] Split health checks into liveness and readiness endpoints.
- [ ] Define production logging and error-reporting expectations.
- [ ] Add appropriate security headers for public pages.
- [ ] Document Railway deployment, S3 setup, and Terraform operations.

## 5. Quality Gates

- [x] Format, Tailwind generation, Rustywind, lint, and test checks exist.
- [x] Run `just check` in CI.
- [ ] Add route-level HTTP tests for standard pages and failures.
- [ ] Cover production asset versioning and Brotli behavior in tests.
- [ ] Add an end-to-end smoke check for the production container.

## 6. Optional Integrations

- [ ] Add self-hosted Umami on Railway with Railway PostgreSQL.
- [ ] Load analytics only when its environment variables are configured.
- [ ] Track meaningful CTA, outbound-link, and footer-link events.
- [ ] Add analytics dashboard goals and funnels before adopting more
      observability.
