# M1 builder handoff

- Work order: `venture-client-action-room-m1`
- Date: 2026-08-28
- Branch: `main`
- Runtime source: `612003b09b2513e79d32df7ed456355c430f5018`
- Production: <https://client-action-room.sociobot.in>
- Milestone state: built and deployed; independent review/polish is still required before M2 starts

## What shipped

- Replaced the planner placeholder with the full municipal-archive landing page, live product preview, three-step explanation, privacy boundary, and route-specific legal and 404 pages.
- Added a one-click, 24-hour isolated sample room for Northline Studio and Alder Street Bakery. A visitor can create an approval, publish a fragment-carried scoped link, answer without an account, and inspect the resulting append-only audit event.
- Added reset and leave-demo operations that destroy the current namespace. Demo tables have no organization, identity, billing, email, upload, blob, or AI path.
- Added SQLite persistence, automatic forward migrations, a checked down migration, hourly demo expiry, SHA-256-only grant storage, HttpOnly scoped client sessions, same-origin mutation checks, server-side validation, and idempotent submissions.
- Added rate limits to every non-health route. Limits use the first `X-Forwarded-For` address and return `429` with integer `Retry-After`. Security headers and redacted structured request logs apply throughout.
- Added light and dark archive treatments, self-hosted Newsreader and Public Sans subsets, hand-authored original SVG art, responsive/mobile layouts, designed focus states, reduced motion, loading/error/offline/empty/expired states, and route focus announcements.
- Added metadata, canonical/OG/Twitter tags, a 1200×630 social image, SVG/favicon/touch icons, `robots.txt`, `sitemap.xml`, and SPA/container fallbacks.
- Kept choice, upload, and external-link rows explicitly labelled as previews. “Start for real” destroys the sample and explains that accounts arrive in M2.

## Claims and evidence

The six predeclared entries in `.factory/claims.json` are now implemented. Each has exactly one `@claim:<id>` Playwright test from a fresh demo:

- `demo-one-click`
- `demo-reset`
- `client-no-account`
- `deadline-order`
- `approval-audit`
- `link-expiry`

The exact claim wording appears in the product and README. Screenshots for landing, demo, client-ready, client-complete, audit, expired, legal, and 404 states are in `.factory/evidence/screenshots/`.

## Verification

Passed from a clean clone at `/tmp/client-action-room-clean.UTnlhd`:

```text
npm ci
npm run check                 0 Svelte errors/warnings; fmt and clippy clean
npm test                      5 Vitest + 2 Rust unit + 4 Rust API tests
npm run test:e2e              7 Playwright tests, including all 6 claims
npm run build                 web dist and release Rust binary produced
```

Additional evidence:

- 100-request load smoke: 60 responses were `429`, each with `Retry-After`; `/health` remained exempt.
- Only-`PORT` startup passed. The application generated its writable SQLite location and applied migrations without a supplied secret.
- Public bundle: 25.56 KB JS gzip, 4.86 KB CSS gzip, 65.4 KB fonts, 1.3 KB hero SVG, 5.2 KB social PNG.
- Lighthouse mobile: performance 99, accessibility 100, best practices 100, SEO 100; LCP 2.1 s, FCP 1.4 s, CLS 0, TBT 40 ms.
- Lowest checked light-theme text contrast: 5.63:1; focus: 4.74:1. Dark-theme values are higher.
- `npm audit`: 0 known vulnerabilities.
- Final ACR container build `chkj` passed. The non-root Azure Container Apps revision starts with only `PORT`, migrated its database, and reports `612003b09b2513e79d32df7ed456355c430f5018` from `/health`.
- A cold custom-domain check returned 200. The factory URL verifier found the correct title, `lang`, one heading, one main landmark, complete image/button names, and zero console errors.
- `PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e` passed all seven browser tests, including all six claims, against production.
- The browser suite supports `PLAYWRIGHT_BASE_URL` so the same claim tests can run against a deployed environment without starting a local server.

## Scope decision

The work-order sentence asking for CIAM, real firm persistence, and billing conflicts with the milestone contract. The plan expressly assigns those features to M2 and says M1 must not fake an account. This build follows the plan: M1 has real persistent storage for isolated demo sessions, but no production firm account or checkout.

Recurring billing also cannot honestly use the attached one-time paid-unlock flow. M2 must use the factory-registered recurring Sociobot entitlement API, with Dodo only behind that service. The plan was correct and did not need a scope change.

## M2 needs

- Register `https://client-action-room.sociobot.in/auth/callback` on SPA client `25c704f4-465a-47af-80ab-2c489466b697`.
- Register Starter ($49/month) and Studio ($99/month) recurring test and production prices, then provide the recurring Sociobot checkout/webhook/verification contract. Do not substitute a one-time license.
- Add Entra PKCE in the browser and full discovery/JWKS JWT checks in the API; key staff by `oid`.
- Add organization/membership/subscription/workspace/action migrations with PostgreSQL-ready tenant isolation and `/ready`.
- Preserve `/demo` as a separate namespace and rerun every M1 claim alongside M2 account, isolation, and billing claims.

There are no known M1 functional gaps. The required independent review/polish PASS has not happened in this builder work order, so M2 must not start until that gate closes.
