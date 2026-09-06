# Client Action Room — M2 builder handoff

Date: 2026-09-06 UTC

## Outcome

M2 is a deployed builder candidate. It adds durable firm accounts, tenant-scoped workspaces, staff roles, owner controls, and an honest recurring-billing boundary while preserving the accepted M1 demo and public behavior.

- Live URL: <https://client-action-room.sociobot.in>
- Implementation SHA: `d82ccfc2d8f27338232192c763e17abcf699d812`
- Implementation commits: `41d9b9d7e404a0aadfac0601cbcc2a0f7a8de51f`, `d82ccfc2d8f27338232192c763e17abcf699d812`
- Documentation and evidence SHA: `00f391988d2ce0683b68cbb1c768674b49cc9d8c`
- Image digest: `sha256:d2a08314e46cbdb3f37b0a95bcd67e99c87449615acca9c31a5d1547d17258ec`
- Live health: `{"status":"ok","build_sha":"d82ccfc2d8f27338232192c763e17abcf699d812"}`
- Live readiness: database and malware scanner report `ready`
- Runtime: one replica with SQLite, generated local secrets, and uploads on the fleet-created `/data` mount
- Review state: fresh independent M2 verification and strict review are required before acceptance

The later handoff commit and its pointer are documentation-only. They do not require a new image; the health build ID above is the implementation classification.

## M2 scope delivered

- Added `/auth/callback`, `/onboarding`, `/app`, `/app/workspaces/:id/actions/new`, `/app/settings`, and `/app/billing` with route-specific titles and the existing site skeleton.
- Added CIAM discovery and cached JWKS loading. Bearer tokens require RS256, the discovered issuer, the configured client audience and tenant, valid time claims, and a known `kid`. The cache refreshes on an unknown key and fails closed.
- Keyed staff identity by stable `oid`. No password or product-owned identity store was added.
- Added organizations, staff profiles, owner/admin/member memberships, invitation digests, workspaces, subscriptions, and deletion scheduling to SQLite.
- Added owner-controlled firm details, retention, JSON export with checksum metadata, exact-name deletion scheduling, cancellation, and a controlled-clock purge at seven days.
- Enforced organization and workspace scope on every staff query. Owner/admin/member permissions and unpaid workspace/seat limits are exercised by API tests.
- Added subscription state and entitlement-backed plan limits without inventing a checkout or accepting a local paid flag.
- Published planned Starter at $49/month and Studio at $99/month. The plans page says checkout is unavailable until the recurring offers are registered.
- Added owner-focused product screens with empty, loading, error, and recovery states. At 390 px and 200% text, the existing demo controls remain visible without horizontal page scrolling.
- Kept demo mode isolated and account-free. Real-workspace reminders now say “Record reminder plan”; neither demo nor M2 claims that an email was sent.

The schema change is `server/migrations/202609060005_accounts_persistence.up.sql` with its matching down migration.

## Claim and regression evidence

`.factory/claims.json` contains 18 public claims, each mapped to one observable Playwright test. From a fresh clone of the implementation SHA, every declared command was run separately and passed.

The three controller-requested public boundaries passed as outcomes:

| Promise | Observed result |
|---|---|
| 5 MB upload boundary | An exact 5 MiB PDF completed the fixture scan; an otherwise valid file one byte larger was rejected with the size recovery message. |
| 24-hour file expiry | A controlled clock returned the uploaded bytes at 23:59:59 and denied access at the exact 24-hour boundary after expiry cleanup. |
| Demo reminders send no email | Scheduling added one reminder audit record while delivery status remained at zero email deliveries and zero real queue entries. |

Other retained regressions passed:

- Rotating `car_demo` cookies shared the intended stable allowance. The live sequence was `201, 201, 201, 429`; the 429 included `Retry-After: 58`.
- Separate server-issued visitor cookies retained independent demo allowances.
- A second firm cannot read another firm's workspace. Durable organization state survives a second app instance and a snapshot/restore test.
- Missing and invalid staff tokens return 401. Owner-only settings reject lesser roles.
- A firm can export its own records, schedule deletion, cancel it, and continue working. At the controlled seven-day boundary the organization and its owned rows are purged.
- The one-click sample, persistent demo label, four realistic actions, reset, and “Start for real” deletion boundary still pass.
- The deliberate missing route renders the product 404 and returns HTTP 404; this is expected behavior.

## Clean-checkout verification

Executed in a fresh clone at `d82ccfc2d8f27338232192c763e17abcf699d812`:

```sh
npm ci
npm test
npm run check
npm run build
npm run test:e2e
```

- `npm ci`: 89 packages, 0 vulnerabilities.
- `npm test`: 5 Vitest tests, 9 Rust unit tests, 10 Rust integration tests, and Rust doc tests passed.
- `npm run check`: Svelte reported 0 errors and 0 warnings; rustfmt and clippy passed with warnings denied.
- `npm run build`: Vite and the locked release Rust build passed. Public entry JS is 98.54 kB (33.28 kB gzip), lazy auth JS is 263.71 kB (67.07 kB gzip), and CSS is 21.30 kB (5.15 kB gzip).
- `npm run test:e2e`: 21/21 local browser tests passed.
- All 18 exact commands in `.factory/claims.json` passed independently.

The Rust API suite includes token validation, tenant and role boundaries, limits, invitation acceptance, export, deletion recovery and purge, reversible migration behavior, cross-instance persistence, snapshot restore, security headers, and rate limiting with `Retry-After`.

## Accessibility and performance

- The factory URL verifier passed `/` and `/app/settings` locally, and `/` live: correct title, language, one H1, main landmark, alt text, labelled controls, and no console errors.
- Playwright axe checks found zero serious or critical issues.
- Standalone `@axe-core/cli` could not run because its ChromeDriver did not match the preinstalled browser and the first launch lacked sandbox flags. The required Playwright axe integration ran against the product and passed.
- Local mobile Lighthouse: performance 100, accessibility 100, best practices 100, SEO 100; LCP 1.63 s, CLS 0, TBT 21 ms.
- Fresh desktop and 390 px phone views name the job, audience, and first action before scrolling. The live 200% phone-text regression passed with no horizontal page scroll.
- Keyboard focus, reduced motion, legal routes, route titles, links, and security headers remain covered by the browser suite.

## Deployment and live verification

The product deployment script built and deployed only `sf-client-action-room` resources. ACR run `ch2cc` completed successfully. The existing `sf-client-action-room-data` share, `/data` mount, probes, environment, and one-replica bound were preserved.

```sh
bash .factory/deploy.sh
curl -fsS https://client-action-room.sociobot.in/health
curl -fsS https://client-action-room.sociobot.in/ready
PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e
```

- Live browser suite: 18 passed and 3 intentionally local-only cases were skipped. The controlled-clock expiry, local signed-token real workspace, and firm-owner fixture all passed locally at the same implementation SHA.
- The exact live upload-size and zero-delivery demo-reminder boundaries passed.
- `/`, `/demo`, `/workspace`, `/onboarding`, `/app`, `/app/settings`, `/app/billing`, `/privacy`, `/terms`, `robots.txt`, and `sitemap.xml` returned 200. The designed unknown route returned the expected 404.
- Internal links returned 200. The explicit email link remained a `mailto:` link.
- A fresh live desktop and phone browser showed the sample entry point without setup or overflow.

## External dependencies and operator actions

These are not shipped-capability claims and do not block the free demo or local M2 verification:

1. **CIAM end-to-end sign-in:** discovery and JWKS are reachable, and an unauthenticated authorize request with `https://client-action-room.sociobot.in/auth/callback` returned the expected sign-in-required response rather than a redirect mismatch. No test identity was available, so a complete hosted sign-in and callback round trip remains for operator-backed verification. Confirm that exact redirect URI on SPA client `25c704f4-465a-47af-80ab-2c489466b697`.
2. **Recurring billing:** `https://api.sociobot.in/api/v1/products/client-action-room/checkout` returned the billing service's not-registered 404. Register monthly Starter ($49, five workspaces) and Studio ($99, fifteen workspaces) offers, then supply and verify the recurring checkout, return, webhook, cancellation, and entitlement-reconciliation contract. No checkout link is exposed meanwhile.
3. **Billing metadata:** `.factory/billing-offer.json` and `/work/.evidence/billing-offer.json` contain the actual Starter offer requested by the brief. They contain no credential. Studio also needs operator registration before either offer is made purchasable.
4. **Email delivery:** M4 owns the transactional email adapter and outbox. Demo reminders intentionally never send; M2 real-workspace UI records a reminder plan only.
5. **Real file/choice/external-link authoring:** these remain M3. Their M1 demo paths stay available and tested, but they are not presented as signed-in M2 capabilities.
6. **Backups:** atomic snapshot/restore behavior is tested. Scheduling and retention of off-site backups remain an operations dependency in M4.

No Azure key, external identity credential, payment credential, or real recipient was used or recorded.

## Evidence files

- Public claims: `.factory/claims.json`
- Demo boundary: `.factory/demo.md`
- Product plan/status: `.factory/plan.md`
- Design and component updates: `.factory/design.md`, `.factory/component-inventory.md`
- Plain-language audit: `.factory/copy-audit.md`
- Billing registration metadata: `.factory/billing-offer.json`
- Catalog copy: `.factory/catalog-description.txt` and `/work/.evidence/catalog-description.txt`

Next action: run fresh independent M2 verification against implementation `d82ccfc2d8f27338232192c763e17abcf699d812`, then strict review. Do not advance M3 or call M2 accepted before both pass.
