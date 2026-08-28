# Independent verification — FAIL

- **Candidate:** `9964a45ded39956d5e222d62528c629ce075930b`
- **Live URL:** <https://client-action-room.sociobot.in>
- **Verified:** 2026-08-28 UTC
- **Verdict:** **FAIL — do not release or begin M2.**

## First read

Cold opening the live landing page answers the required questions in plain words. It says that Client Action Room gets client actions done on time, is for small firms chasing approvals/files/choices/payment links across email, and makes **“Try it with sample data”** the first action. The button says that it opens a ready room in one click and that nothing is saved to an account. This part passes.

## Required claim tests from a clean clone

I cloned the candidate cleanly to `/tmp/client-action-room-qa.M7Xw8x`, checked out the candidate SHA, ran `npm ci`, then ran every command listed in `.factory/claims.json` before other product tests. All six passed locally against the shipped demo entry point:

| Claim | Exact command | Local result |
|---|---|---|
| demo-one-click | `npm run test:e2e -- --grep @claim:demo-one-click` | PASS |
| demo-reset | `npm run test:e2e -- --grep @claim:demo-reset` | PASS |
| client-no-account | `npm run test:e2e -- --grep @claim:client-no-account` | PASS |
| deadline-order | `npm run test:e2e -- --grep @claim:deadline-order` | PASS |
| approval-audit | `npm run test:e2e -- --grep @claim:approval-audit` | PASS |
| link-expiry | `npm run test:e2e -- --grep @claim:link-expiry` | PASS |

`.factory/claims.json` is present and has six correctly tagged tests. The complete local Playwright suite also passed: 7/7.

## Release-blocking findings

### P0 — live demo sessions lose their state between requests

**Reproduction (fresh evidence):** Against the live candidate, twenty times, `POST /api/v1/demo/session/ensure` returned `201` and supplied a new `car_demo` cookie. The immediately following authenticated `GET /api/v1/demo/queue` returned `410` with `{"code":"demo_expired","message":"This sample room has expired. Reset the demo to open a fresh copy."}` in **20/20** cases. The source uses an SQLite file under `/data`; the observed production behaviour is consistent with requests reaching instances without a shared persistent database.

The independent live command `PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e` consequently failed 3/7 tests:

- `@claim:client-no-account`
- `@claim:approval-audit`
- `@claim:link-expiry`

Their retained Playwright evidence is in the clean clone at `.factory/evidence/test-results/`; each displays the same live `demo_expired` error. A demo that cannot reliably publish or complete a client action is not a usable one-click demo and fails the declared claims in production.

### P0 — candidate does not deliver the brief’s smallest useful product

The researched brief requires branded action links with a deadline queue, secure uploads, approval forms, payment/booking links, reminders, and staff audit trail. This candidate deliberately ships only an **isolated M1 approval demo**. Its upload, choice, and invoice rows explicitly say “Preview only”; there is no secure upload or malware scanning, real choice flow, payment/booking-link flow, reminders, staff account/workspace, CIAM sign-in, subscription, or durable real-firm data boundary. The landing page confirms it “does not … collect payments,” and the README says these capabilities remain later milestones.

The narrow approval demo is useful as a prototype, but it does not meet the product contract’s end-to-end job-to-be-done or the brief’s privacy/security constraints. It must not be accepted as the product release.

## What passed

- Clean-clone gates: `npm test` (5 web + 6 API tests), `npm run check` (Svelte check, rustfmt, clippy `-D warnings`), `npm run build` (Vite production `dist/` plus Rust release build), and local `npm run test:e2e` (7/7) all passed.
- Live `/health` reports `{"status":"ok","build_sha":"9964a45ded39956d5e222d62528c629ce075930b"}`. The live JS and CSS SHA-256 values exactly match the clean candidate production build, so this is not a stale deployment.
- Privacy/network: Playwright request logs for cold landing and demo flows contained only same-origin requests; no analytics or third-party runtime requests were observed. Normal cold load had no console or page errors.
- Accessibility/responsiveness: desktop and 390 px mobile landing/demo/privacy/terms/404 routes each had one `<h1>` and one `<main>`, no horizontal overflow, and no axe serious/critical findings. Keyboard skip-link focus is a visible `rgb(23,110,137)` 3 px outline; reduced motion resolves to `0.00001s`. Keyboard-only local error/recovery checks passed: blank composer, missing decision, and missing change note announce clear errors; focus moves to `#approval-error`; a corrected change request records successfully.
- Security/caching: live responses set CSP with `frame-ancestors 'none'`, HSTS, `nosniff`, `no-referrer`, COOP, and a restrictive permissions policy. API/client responses are `no-store`; hashed assets are `public, max-age=31536000, immutable`.
- Rate limiting: with one client identity, 100 concurrent unauthenticated `GET /api/v1/demo/queue` requests produced 40 `401` responses then 60 `429` responses, each with `Retry-After: 1`. Observed allowance: **40 reads per second per client**. This passes the server rate-limit requirement.
- Performance artifacts: initial JS is 25,453 bytes gzip (68,989 bytes raw); CSS is 4,852 bytes gzip; shipped WOFF2 fonts total 65,444 bytes. All are within the stated budgets.

## Limits of this verification

`docker` is not installed in the disposable verification container, so the Docker image build/runtime health smoke could not be run here. This is environment-unavailable evidence, not a candidate pass. The repository’s exact production build command (`npm run build`) did pass.

## Required next steps

1. Fix deployment persistence before retesting: use a shared durable database/service, or explicitly guarantee a single instance plus durable storage and prove session affinity/persistence. Then rerun all declared claims against the live URL from fresh contexts.
2. Complete the actual brief before product acceptance: implement secure uploads with scanning/expiry, choices, HTTPS payment/booking links, reminders, staff audit/workspace persistence, CIAM for staff, and subscription via Sociobot billing; test each as a claim in the demo sandbox.
3. Retain the existing local quality/accessibility/privacy/rate-limit checks, then repeat independent live verification.
