# Independent verification 2 — FAIL

- **Candidate:** `052905494eff58fe50bab41ec254b956a68ae353`
- **Live URL:** <https://client-action-room.sociobot.in>
- **Verified:** 2026-08-28 UTC
- **Verdict:** **FAIL — do not release this candidate.**

This is a fresh verification, not a reliance on the builder's deployment report. The live `/health` response identified exactly `052905494eff58fe50bab41ec254b956a68ae353`; the deployed main JS and CSS have the same SHA-256 values as this candidate's fresh production build.

## First read

Cold-opening the live landing page passes the mandatory first-screen check. It says **“Get client actions done on time”**, names **small firms chasing approvals, files, choices, and payment links across email**, and makes **“Try it with sample data”** the primary action. The adjacent copy says a ready action room opens in one click and nothing is saved to the visitor's account. The action is one click to `/demo`.

## Required claim gate

`.factory/claims.json` is present and contains ten entries. From this clean candidate checkout, after `npm ci`, I ran every declared command against the product demo entry point. All ten passed. The consolidated local run (`npm run test:e2e -- --grep '@claim:'`) also passed 10/10, and the full local suite passed 12/12.

| Claim | Result |
|---|---|
| `demo-one-click` | PASS |
| `demo-reset` | PASS |
| `client-no-account` | PASS |
| `deadline-order` | PASS |
| `approval-audit` | PASS |
| `link-expiry` | PASS |
| `secure-upload` | PASS for the shipped EICAR fixture only; see P0-3 |
| `choice-flow` | PASS |
| `external-link` | PASS |
| `reminder-audit` | PASS |

The fresh live run, `PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e`, also passed 12/12. This proves the current demo works, but does not cure the release blockers below.

## Release-blocking findings

### P0-1 — live rate limiting is not enforced

The backend-service contract requires every server endpoint to return `429` with `Retry-After` once one client exceeds its allowance. Fresh production probes from one verifier process did not reach a limit:

- 100 concurrent unauthenticated `GET /api/v1/demo/queue` requests returned **100 × 401**, **0 × 429**. The source policy documents 40 reads per second.
- Five sequential `POST /api/v1/demo/sessions` requests returned **5 × 201**, **0 × 429** and no `Retry-After`. The source policy documents 3 demo-session writes per minute.

The probes supplied the same `X-Forwarded-For` value; the ingress may rewrite that header, but the observable public API still failed to enforce a documented allowance for one client process. **Observed live allowance: at least 100 unauthenticated read requests in a burst and at least 5 demo-session writes/minute; no limit or retry interval could be observed.** This is a direct release-blocking contract failure.

### P0-2 — no real firm product exists behind “Start for real”

The researched brief requires a real branded portal for agencies: staff-created actions, durable firm workspaces, scoped client links, reminders, audit trail, and eventual recurring billing. This candidate only provides the sample room.

Fresh source evidence:

- `src/App.svelte:86` explicitly says: **“Real accounts and monthly plans are not available in this release.”**
- The signed-in `/workspace` flow calls `/api/v1/me`, then reads `/api/v1/demo/queue` (`src/App.svelte:166-169`).
- `server/src/lib.rs:93-94` provisions the signed-in staff user with `demo::provision_staff` and sets the `car_demo` cookie.
- `server/src/demo.rs:969-991` maps the Entra `oid` to a 10-year `demo_sessions` record. `server/src/demo.rs:996-1099` seeds every such workspace with **Northline Studio / Alder Street Bakery / Theo Grant / Maya Chen**, the sample actions, and `https://example.com/`.
- All persistence migrations and real-path action endpoints are named `demo_*` or `/api/v1/demo/*`; the only create form is **“Create another approval.”** There are no organization, workspace, non-demo action, audit-export, retention, billing, or production file-storage routes.

This is not a small gap in a real portal: it is a sandbox being relabelled as a firm workspace. The unauthenticated demo is a good trial, but it cannot satisfy the real job-to-be-done for an agency or the web-with-backend product contract.

### P0-3 — “safety scan” is not malware scanning

The brief requires malware scanning and the live product claims a client PDF is “safety-scanned.” `server/src/demo.rs:720-816` accepts any byte sequence beginning `%PDF-`, rejects only a case-insensitive five-byte `EICAR` substring, then writes the content and a `scan_state` of `clean`. There is no scanner adapter, quarantine workflow, asynchronous scan, signature/AV engine, or fail-closed unavailable-scanner behavior.

The declared `secure-upload` test therefore proves only that its artificial EICAR fixture is rejected. A different malicious PDF is accepted and represented to the client/staff as clean. This is unsafe and materially misstates a security property.

### P1-1 — unlisted user-facing claims violate the claims contract

The claim list tests the ten named demo outcomes, but the landing page and README make further visitor-reliant claims without an entry/test. Examples include:

- Landing: “The sample uses a temporary server-side room. Reminder actions are recorded but no sample email is sent.”
- Privacy page: no analytics, no advertising, no email delivery, fragment-only secrets, one-way digests, reset/leave deletion, and hourly expiry purging.
- README: demo-router isolation from organization/billing/email/AI services; token secrecy; SQLite digest-only storage; and CIAM validation details.

Some claims may be true, and request logging did show same-origin-only runtime requests on cold landing and demo flows, but the contract requires each claim a visitor could rely on to have exactly one observable sandbox test. These statements are not in `.factory/claims.json` and are release blockers until tested or removed.

## What passed

- Clean installation: `npm ci` installed 89 packages with 0 reported vulnerabilities.
- Tests: `npm test` passed (5 web tests; 10 Rust unit/integration tests). `npm run check` passed Svelte diagnostics (0 errors/warnings), rustfmt, and clippy with `-D warnings`. `npm run build` produced `dist/` and the Rust release binary.
- Browser flows: local 12/12 and live 12/12 Playwright suites passed, including the ten claims, reset/isolation, scoped approval/upload/choice/external flows, reminder audit, keyboard smoke, routing, privacy-request smoke, and axe serious/critical checks.
- Accessibility/responsiveness: a fresh axe scan of `/`, `/demo`, `/privacy`, `/terms`, `/workspace`, and the 404 page at desktop and 390×844 found no serious/critical violations. Every route had one `<h1>` and one `<main>` and no horizontal overflow. Keyboard focus on mobile was a visible `rgb(23, 110, 137)` 3 px outline. Under reduced motion the measured transition duration was `0.00001s`.
- Privacy/network: cold live landing requests were all same-origin (fonts, theme, app JS/CSS, and the self-hosted illustration), with no console/page errors. Demo flow request logging was also same-origin-only. No third-party font or runtime script was requested.
- Security headers/caching: live root, API, and asset responses set CSP including `frame-ancestors 'none'`, HSTS, `nosniff`, `no-referrer`, COOP, and Permissions Policy. API responses are `no-store`; hashed assets are `public, max-age=31536000, immutable`.
- Performance budget: first-route JS is 79,551 bytes raw / 28,421 bytes gzip; CSS is 19,367 bytes raw / 4,883 bytes gzip; self-hosted WOFF2 fonts total 65,444 bytes. These are within the stated public-route budgets. The 67 KB gzip auth chunk is lazy-loaded.
- Candidate parity: live `assets/index-B0yL1BGT.js` SHA-256 is `2afbd7a84ced9cd8a6c73e3d5b79285f902f8f31760da2046f560857edbaedda`, exactly equal to `dist/assets/index-B0yL1BGT.js`; live CSS likewise exactly equals the candidate build.

## Verification limits

`docker` is not installed in this disposable verifier container, so the Docker image build/runtime smoke could not be run. The repository's exact production build did pass. I did not authenticate a real Entra user because no test identity was provided; the public sign-in configuration uses the required `sociobotcustomers.ciamlogin.com` authority, but its signed-in product behavior is demonstrably only a seeded demo from source review.

## Required next steps

1. Fix ingress/runtime rate-limit identity and prove each endpoint returns `429` plus `Retry-After` after its documented allowance.
2. Implement the actual signed-in firm product with organization/workspace ownership, non-demo data model/routes, configurable branded actions, durable audit export/retention, real reminder delivery controls, and Sociobot recurring entitlement before marketing “Start for real.”
3. Replace the EICAR substring heuristic with a fail-closed malware-scanning/quarantine adapter. Do not call files clean until a scanner returns a clean result.
4. Add observable sandbox tests for every remaining privacy/security/lifecycle claim, or remove those claims from public copy and README.
5. Repeat a full independent live verification after the above changes.
