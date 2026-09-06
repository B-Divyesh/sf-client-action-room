# Strict review 1 — Client Action Room

- **Candidate implementation reviewed:** `87e07dcd5a3b95eac9e1a71a1d42456614440757`
- **Documentation / checkout SHA reviewed:** `ac87951892ed76d1b5c2c1b14d0a19f3e4ab77a1`
- **Live build reported by `/health`:** `b1e96e09fe403c15a2626e92a7603e51ecd1715a`
- **Live URL:** <https://client-action-room.sociobot.in>
- **Reviewed:** 2026-09-06 UTC
- **Current milestone:** M1 repair 3, strict review after Independent Verification 4
- **Verdict:** **FAIL — 2 findings, 0 untested claims. Do not accept M1 yet.**

The commits after `87e07dc` contain browser-test wait changes and documentation only. A fresh build from `ac87951` produced the same main JavaScript and CSS bytes as live, so `87e07dc` remains the implementation candidate and `ac87951` is the reviewed documentation SHA.

## First screen and sample boundary

Before scrolling, fresh desktop (1280×900) and phone (390×844) contexts both showed:

- **Job:** Get client actions done on time.
- **Audience:** Small firms chasing approvals, files, choices, and payment links across email.
- **First action:** **Try it with sample data**.

The action was visible at `scrollY = 0` in both contexts. It opened the four-action Northline Studio / Alder Street Bakery sample in one click. The persistent label read **“Demo — sample data, nothing is saved”**. Reset restored all four actions on desktop and phone. Review activity used isolated demo rooms only; no real workspace was opened or changed.

## Findings

### P0 — rotating the demo-session cookie bypasses the live write allowance

The public session-creation endpoint does not enforce its documented three-writes-per-minute allowance for a direct API client that follows the cookie returned by the endpoint.

Fresh live reproduction from one process and network identity:

```text
POST /api/v1/demo/sessions, accepting and returning the car_demo cookie
201, 201, 201, 201, 201
Retry-After: absent on every response
```

Control reproduction with a fixed `car_visitor` cookie:

```text
POST /api/v1/demo/sessions
201, 201, 201, 429, 429
Retry-After: 59 on both 429 responses
```

`server/src/lib.rs` chooses the first available cookie identity and returns immediately. A direct API client starts without `car_visitor`, so its first request uses the anonymous bucket and later requests use `car_demo`. Each successful session creation replaces `car_demo`, selecting a new limiter bucket on the next request. The client IP identities are not checked once any product cookie is present. The local regression test changes forwarded IP values but does not carry the newly issued `car_demo` cookie, so it misses the public bypass.

This violates the mandatory backend contract that every server endpoint enforce a client allowance and return `429` with `Retry-After` past it. Unlimited session creation also exposes storage and scanner capacity to avoidable abuse. Fix the limiter so a stable IP/visitor identity remains enforced even when session cookies rotate, add an observable regression test that follows `Set-Cookie`, and repeat the live probe.

### P1 — 200% text sizing forces horizontal page scroll on a phone

At a 390×844 phone viewport, increasing the root text size from 16 px to 32 px caused the demo page width to grow from 390 px to 640 px. The app root, demo banner, header, main content, and controls all extended beyond the viewport.

The cause is the root `min-width: 20rem` in `src/styles/app.css`; the minimum scales to 640 px with 200% text sizing. This conflicts with the accessibility contract and the visual thesis requirement that 200% text zoom reflow without horizontal page scroll. Remove the scalable page minimum or replace it with a reflow-safe constraint, then add a 200% text-size browser check at 390 px.

## Declared claims

From a clean clone of `ac87951`, `npm ci` installed 89 packages with zero reported vulnerabilities. Every exact command in `.factory/claims.json` was then run separately. All 16 passed, so there are **zero untested claims**.

| Claim | Result | Outcome exercised |
|---|---|---|
| `demo-one-click` | PASS | Landing action opened the seeded four-action room. |
| `demo-reset` | PASS | Isolated changes did not cross sessions; reset restored the seed. |
| `client-no-account` | PASS | A signed-out client completed an approval. |
| `deadline-order` | PASS | Open actions appeared in ascending deadline order. |
| `approval-audit` | PASS | Decision, actor, and controlled server time were recorded. |
| `link-expiry` | PASS | Seven-day expiry denied read and submit. |
| `secure-upload` | PASS | Type, scope, EICAR rejection, and clean PDF paths passed. |
| `upload-5mb-boundary` | PASS | Exactly 5 MiB was accepted; 5 MiB plus one byte was rejected. |
| `file-expiry` | PASS | Bytes were readable at 23:59:59 and denied at 24:00:00 after purge. |
| `choice-flow` | PASS | The selected crop reached the audit. |
| `external-link` | PASS | Destination disclosure and open-only audit passed. |
| `reminder-audit` | PASS | The scheduled reminder and staff actor reached the audit. |
| `demo-reminders-no-email` | PASS | One schedule produced zero delivery-queue entries and same-origin traffic. |
| `real-workspace` | PASS | The local auth fixture proved an empty durable workspace and owner isolation. |
| `demo-privacy` | PASS | Same-origin traffic, fragment removal, 24-hour maximum, and leave deletion passed. |
| `staff-auth` | PASS | Missing and invalid tokens returned 401 with `WWW-Authenticate: Bearer`. |

The full local browser suite passed 18/18. The live public suite passed 16 tests; only the controlled-clock expiry and local test-identity workspace fixture were skipped live, and both passed locally.

## Other verification evidence

Clean-checkout commands:

```sh
npm ci
npm test
npm run check
npm run build
npm run test:e2e
PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e
```

- `npm test`: 5 web tests, 6 Rust unit tests, and 8 Rust integration tests passed.
- `npm run check`: Svelte reported 0 errors and 0 warnings; rustfmt and clippy passed.
- `npm run build`: produced `dist/` and the release server. Initial JS was 28.91 KiB gzip; CSS was 4.91 KiB gzip.
- Asset parity: local/live JS SHA-256 `f09566501f9bd6004f5804e53f30e1e508e13c9c6fabdc7237f4f2a99ecb3814`; local/live CSS SHA-256 `bd3e197c681895e9483269a1aaef5bfff2a883bd68393c3fb1e09c02df9c7cb9`.
- `/health` and `/ready` returned 200; database and malware scanner reported ready.
- The supplied `verify-url.sh` passed: title, `lang=en`, one h1, one main, image alt coverage, named buttons, and no normal-load console errors.
- Playwright axe found zero serious or critical violations across `/`, `/demo`, `/privacy`, `/terms`, `/workspace`, and the designed 404 in desktop/phone and light/dark combinations.
- `npx @axe-core/cli` was attempted but its Selenium wrapper could not find a Chrome binary. The installed Playwright Chromium completed the equivalent axe scans above.
- Lighthouse mobile: performance 100, accessibility 100, best practices 100, SEO 100, LCP 1.4 s, CLS 0, total transfer 99 KiB.
- Reduced-motion contexts reported `0.00001s` transitions. Keyboard skip-link and route-heading focus passed. Blank approval and missing-note errors were announced and focused `#approval-error`; correction completed successfully.
- Normal routes returned 200 with distinct titles and no console errors. The unknown route intentionally returned HTTP 404 with one h1, one main, a designed recovery link, and the expected browser 404 resource message; this is not a defect.
- Internal links, legal pages, `robots.txt`, `sitemap.xml`, favicon, security headers, self-hosted assets, and same-origin demo traffic passed. The product makes no offline or update promise.
- Docker is unavailable in this worker, so no fresh image smoke was possible. The clean native build passed; this environment limit is not classified as a product finding.

## Earlier findings and current disposition

| Earlier item | Strict-review disposition |
|---|---|
| Demo state lost between live requests | Fixed. Live public flows pass; cross-instance and snapshot/restore integration tests pass. |
| Incomplete product scope | Fixed for current M1 scope. Demo action flows and the real approval loop pass; later milestone work is not advertised as shipped. |
| Live rate limiting absent | **Reopened.** Stable browser visitors receive 429, but rotating the endpoint-issued demo cookie bypasses the allowance. |
| “Start for real” used seeded demo data / no owner isolation | Fixed for M1 approval scope by the empty-workspace and other-owner denial claim. |
| Upload scan was a byte heuristic | Fixed. Scanner readiness, clean scan, EICAR rejection, and fail-closed tests pass. |
| Preview-only upload, choice, external link, and reminder | Fixed in the demo; each outcome reaches the audit. |
| Unlisted privacy/security/lifecycle promises | Fixed for the current public scope; all 16 declared commands passed. |
| Verification 3: 5 MiB, 24-hour file expiry, and no-email reminder promises untested | Fixed; all three dedicated outcome commands passed. |
| First-screen clarity, sample label/reset, invalid recovery, focus, reduced motion, route titles, legal pages, designed 404, privacy traffic | Passing, except for the newly found 200% text reflow defect above. |

## Milestone and external dependencies

This review covers **M1 repair 3**. It does not demand future venture capabilities early.

External dependencies and later work remain separate from the two findings:

- Operator confirmation of the shared Entra redirect URI.
- Factory registration of recurring Sociobot prices and entitlement endpoints for M2.
- M2 membership, export/delete, retention controls, subscription enforcement, and backup operations.
- M3/M4 real-workspace upload, choice, and external-link authoring plus opted-in transactional reminder delivery. Demo reminders intentionally create no delivery entry.

M1 must return to fresh independent verification and strict review after both findings are repaired.
