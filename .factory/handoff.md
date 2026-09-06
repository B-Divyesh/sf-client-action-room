# Client Action Room — repair 2 handoff

Date: 2026-09-06 UTC

## Result

M1 repair 2 is implemented, deployed, and locally and live verified. Independent re-verification is still required before the milestone can be called accepted.

- Live URL: <https://client-action-room.sociobot.in>
- Deployed implementation SHA: `b059adf1d4b08f755a2d1a08d3a77e2052586648`
- Clean-checkout test SHA: `c567e9448f313f9b6ed46a00d8f89c1105ea415f`
- Documentation and evidence SHA: `92368fcdb35e73425aba44979bd362c2d30f713e`
- Deployed image digest: `sha256:6bccd857ee5ed43e345057962ceaa486b9f4b6b5d9dec26cd4a02945dc6be8c8`
- Active revision: `sf-client-action-room--0000014`
- Runtime: one replica, 1 CPU, 2 GiB; local SQLite working copy with atomic snapshots to the fleet-mounted `/data` share

The implementation and test SHAs differ because `c567e94` only extends browser waits for real scanner and live network outcomes. It does not change the shipped application. `92368fc` adds the final operational documentation and deployment-script guard.

## Recorded findings and disposition

1. **Live rate limiting — fixed.** Browser sessions use a stable first-party cookie or bearer identity. Anonymous API requests retain a shared fallback, so changing ingress addresses does not bypass the limit. Every 429 includes `Retry-After`. A live burst returned 40 responses before 60 rate-limited responses; strict session creation returned `201, 201, 201, 429, 429`, with `Retry-After: 59`.
2. **Seeded “Start for real” workspace — fixed for M1.** Demo and real workspaces now use separate namespaces. A signed-in owner gets a new, empty organization-owned workspace, can create and publish an approval, and can receive a client decision in its audit record. A different owner is denied. Leaving the demo deletes its isolated room before sign-in.
3. **Malware scan was a byte check — fixed.** Uploads enter a temporary quarantine file, pass PDF validation, and are scanned by ClamAV before any completion or audit record is saved. Timeout, unavailable scanner, and suspicious results fail closed. The production image includes current signatures. The EICAR fixture is rejected and a safe PDF completed live in 16.7 seconds.
4. **Untested public privacy and security claims — fixed.** `.factory/claims.json` now declares 13 outcome tests. They cover the demo boundary, account-free client flow, deadline order, audits, exact link expiry, upload scanning, choices, external destinations, reminders, empty real-workspace isolation, same-origin demo traffic and deletion, and staff-token rejection.
5. **Live demo state loss from the first verification — fixed.** The fleet mount remains the durable source, while the single replica restores a local SQLite working copy and atomically snapshots successful writes. A forced live replica restart retained the same session's reminder audit record.
6. **Preview-only M1 action types from the first verification — fixed.** Upload, choice, external-link, and reminder actions now have complete sample client flows and audit outcomes. The M1 real-workspace approval loop is also usable; later signed-in action types and billing remain scoped to later venture milestones.

Earlier minor findings were also checked: the first screen names the job and audience; sample mode remains visibly labelled; reset restores four actions; phone, keyboard, focus, reduced motion, route titles, legal pages, security headers, intentional 404, and same-origin privacy behavior pass. Historical reports remain in `.factory/verification-2.md` and earlier handoffs.

## Verification evidence

From a clean clone at `c567e94`:

```sh
npm ci
npm test
npm run check
npm run build
npm run test:e2e
```

- `npm ci`: 89 packages, 0 vulnerabilities.
- `npm test`: 5 frontend unit tests, 6 Rust unit tests, and 8 Rust integration tests passed.
- `npm run check`: Svelte check, rustfmt, and clippy passed with no errors or warnings.
- `npm run build`: passed. Initial JS is 28.90 KiB gzip and CSS is 4.91 KiB gzip.
- `npm run test:e2e`: 15/15 passed locally.
- Every command in `.factory/claims.json` was then run separately from that clean clone; all 13 passed.

Against the deployed product:

- Full browser suite: 14 passed and 1 skipped. The skipped real-workspace test needs the local authentication fixture, which is intentionally disabled in production. Token rejection and every public client/demo outcome passed live.
- `/health` reports build `b059adf1d4b08f755a2d1a08d3a77e2052586648`; `/ready` reports the database and malware scanner ready.
- Live JS and CSS hashes match the deployed candidate assets.
- A forced restart changed the replica ID, then the same demo session still contained its scheduled-reminder audit record.
- Desktop at 1280×900 and phone at 390×844 show “Get client actions done on time,” the audience, and “Try it with sample data” before scrolling. The demo showed four realistic actions, a persistent sample label, and restored all four after reset.
- The factory URL check passed with one title, one H1, one main landmark, no missing alt text, no unlabeled buttons, and no console errors on normal routes.
- Axe found zero serious or critical issues on `/`, `/demo`, `/privacy`, `/terms`, `/workspace`, and the designed missing-page route at desktop and phone widths. The only console resource error came from the deliberate HTTP 404 itself.
- Reduced motion collapses transitions to `0.00001s`. There was no horizontal overflow at 390 px.
- Lighthouse mobile: performance 99, accessibility 100, best practices 100, SEO 100, LCP 1.4 s, CLS 0, total transferred weight 99 KiB.
- `/`, `/demo`, `/privacy`, `/terms`, and `/workspace` return 200. An unknown route deliberately returns the designed 404 page.
- CSP, HSTS, `X-Content-Type-Options`, `Referrer-Policy`, `Permissions-Policy`, COOP, and immutable asset caching are present.

The scanner was also exercised locally in production mode with current ClamAV signatures. A normal PDF passed an actual `clamscan`; EICAR returned 422. Browser tests use the explicit `MALWARE_SCANNER_MODE=fixture` mode so a clean checkout stays deterministic.

## Run and deploy

```sh
npm ci
npm test
npm run check
npm run build
npm run test:e2e
bash .factory/deploy.sh
PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e
```

`.factory/deploy.sh` uses only `sf-client-action-room`, the product image, and its fleet-created `/data` mount. It applies snapshot storage, waits for the new healthy revision, and deactivates older product revisions so SQLite has one writer.

## Current milestone and remaining work

M1 now ships the full approval loop, plus demonstrable upload, choice, external-link, and reminder samples. These items remain future milestones or external dependencies and are not presented as working:

- The operator must confirm the shared Entra SPA redirect URI `https://client-action-room.sociobot.in/auth/callback`. The app contains no product password store.
- The researched offer remains a recurring $49/month Starter plan for five active client workspaces. Billing registration and recurring entitlement work belong to M2. No public checkout is advertised, so no billing-offer file was invented.
- M2 still needs owner export/delete and retention controls, staff membership, subscription enforcement, and backup operations.
- Real-workspace authoring currently completes the M1 approval loop. Real upload, choice, and external-link authoring move from the demo into signed-in workspaces in M3.
- Reminder scheduling records an audit event but does not send email. Transactional delivery and its opt-in controls are M3/M4 work.
- ClamAV cold scans are materially slower than the fixture; the live safe-file check took 16.7 seconds at the current 1 CPU/2 GiB allocation.
- The product makes no offline promise. Offline/update behavior is therefore not a shipped claim.

The catalog description is a verb-first 93-character line and is copied to `/work/.evidence/catalog-description.txt` during handoff.

## Verification 3 update — 2026-09-06 UTC

Independent verification reviewed implementation `b059adf1d4b08f755a2d1a08d3a77e2052586648` and the later documentation commit `d4dd83acf0fea73c3da8b04274b4cf64d35f9547`. The live assets exactly matched a fresh candidate build; the live health build id was the documentation SHA.

The verifier passed all local gates, all 13 declared claim commands, the live public browser suite, accessibility checks, rate limits, and a live restart-persistence check. The verdict is nevertheless **FAIL** in `.factory/verification-3.md`: three public claims lack observable tagged tests (the 5 MB upload boundary, 24-hour uploaded-file expiry, and no-email demo reminders). M1 must not be marked accepted until those claims are tested or removed/narrowed. M2 and external dependencies remain as listed above.

## Repair 3 update — 2026-09-06 UTC

### Result

The three Verification 3 claim-coverage findings are repaired and deployed. M1 is ready for fresh independent verification; it is not marked accepted by this handoff.

- Deployed implementation SHA: `87e07dcd5a3b95eac9e1a71a1d42456614440757`
- Later test-only commits: `21f91c4391b347f54d50f5006308355a75f648b1`, `9c176dd78c80ec5b2f105f12f42ff10e1847c784`
- Live health: `{"status":"ok","build_sha":"87e07dcd5a3b95eac9e1a71a1d42456614440757"}`
- Live readiness: database and malware scanner both report `ready`.

### What changed

- Added `upload-5mb-boundary`: a valid 5 MiB PDF completes a real fixture scan; an otherwise valid 5 MiB plus one byte PDF shows the size error.
- Added `file-expiry`: a controlled local demo clock reads the uploaded bytes one second before 24 hours, advances to the exact boundary, purges expired records, and proves the same access cannot return bytes.
- Added `demo-reminders-no-email`: scheduling records one reminder while the isolated delivery-status endpoint reports zero delivery-queue entries. The browser request log remains same-origin.
- Added expiry cleanup for upload records and a staff-demo-only clean-file read endpoint. Files are served only while their demo workspace and 24-hour retention window are active.
- Corrected the size recovery copy to say “no larger than 5 MB,” which matches an accepted exact-limit file.
- Hardened the browser setup helpers to wait for the seeded action queue before direct demo API assertions. This fixes a live timing race without changing the deployed product behavior.

### Verification

From a clean dependency install:

```sh
npm ci
npm run check
npm test
npm run test:e2e
npm run build
```

- `npm run check` passed: Svelte diagnostics, rustfmt, and clippy have no errors or warnings.
- `npm test` passed: 5 web tests, 6 Rust unit tests, and 8 Rust integration tests.
- `npm run test:e2e` passed: 18/18 local browser tests.
- `npm run build` passed and produced `dist/` plus the release server binary. Public JS is 28.91 KiB gzip and CSS is 4.91 KiB gzip.
- Every one of the 16 commands declared in `.factory/claims.json` was run separately from this setup and passed.

The three new claim commands and outcomes were:

```sh
npm run test:e2e -- --grep @claim:upload-5mb-boundary
npm run test:e2e -- --grep @claim:file-expiry
npm run test:e2e -- --grep @claim:demo-reminders-no-email
```

All three passed. The expiry check uses the local fixed clock only; production deliberately exposes no clock control.

Against `https://client-action-room.sociobot.in`:

```sh
PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e
```

The public suite passed with 16 public tests; the controlled-clock file-expiry test and the local-auth real-workspace fixture are intentionally skipped outside the local sandbox. The live suite exercised the exact 5 MiB scan boundary, reminder delivery-status boundary, desktop/phone accessibility smoke, keyboard/focus behavior, designed 404, route titles, same-origin traffic, and axe serious/critical checks. A fresh desktop and 390 px phone load showed the job, audience, and “Try it with sample data” before scrolling.

### Known gaps and next steps

- Fresh independent verification and strict review are still required before M1 acceptance.
- M2 needs operator confirmation of the shared Entra callback and factory registration of recurring Sociobot prices/entitlement endpoints.
- Real-workspace upload/choice/external-link authoring and actual transactional reminder delivery remain M3/M4 scope. Demo reminders intentionally never deliver mail.
- The product makes no offline or update promise.

## Verification 4 update — 2026-09-06 UTC

Fresh independent verification **PASSed** M1 repair 3 with zero findings and zero untested declared claims. The complete report is `.factory/verification-4.md`.

- Candidate implementation reviewed: `87e07dcd5a3b95eac9e1a71a1d42456614440757`.
- Documentation/report SHA reviewed: `b1e96e09fe403c15a2626e92a7603e51ecd1715a`.
- Live `/health` reported the later documentation SHA; fresh local `dist` JS/CSS hashes matched the live assets exactly.
- Clean local gates passed: `npm ci`, `npm test`, `npm run check`, `npm run build`, and 18/18 local E2E tests.
- Every one of the 16 declared claim commands was independently run and passed. This includes exact 5 MiB acceptance/one-byte rejection, controlled 24-hour upload expiry, and demo reminder status with one scheduled reminder and zero delivery-queue entries.
- The live public browser suite passed 16 tests; the controlled-clock expiry and local-auth fixture remain intentionally local-only and passed there.
- Fresh live checks verified the one-click four-action sample, persistent demo label/reset, desktop and phone first screens, invalid/recovery messages and focus, health/readiness, a `201,201,201,429,429` session allowance with `Retry-After: 59`, route titles, legal routes, expected designed 404, same-origin product traffic, security headers, and asset parity.
- The supplied URL verifier and Playwright axe scans passed at desktop and phone widths. `@axe-core/cli` itself could not locate a Chrome binary in this container; this was an environment limitation, while the installed Playwright Chromium scans found no serious or critical violations.

M1 may now proceed to strict review/acceptance. Future work and external dependencies are unchanged: Entra redirect registration and recurring billing registration for M2; membership/export/delete/retention/backups in M2; and real-workspace non-approval action authoring plus actual opted-in reminder delivery in M3/M4. These are not presented as shipped M1 capabilities.

## Strict review 1 update — 2026-09-06 UTC

Strict review **FAILed** M1 repair 3 with two findings and zero untested claims. The complete report is `.factory/review-1.md`.

- Candidate implementation: `87e07dcd5a3b95eac9e1a71a1d42456614440757`; reviewed documentation SHA: `ac87951892ed76d1b5c2c1b14d0a19f3e4ab77a1`.
- All 16 declared claim commands passed independently from a clean clone. Local E2E passed 18/18; the live public suite passed 16 with the controlled-clock and local-auth fixtures skipped live and passed locally.
- P0: a direct client that follows the rotating `car_demo` cookie can create at least five demo sessions without 429. A fixed `car_visitor` receives `201, 201, 201, 429, 429` with `Retry-After: 59`. The stable IP/visitor allowance must remain enforced when session cookies rotate.
- P1: 200% root text sizing at 390 px expands the page minimum from 320 px to 640 px, creating horizontal page scroll. The root `min-width: 20rem` must be made reflow-safe and covered by a browser test.
- Other checks passed: clean tests/check/build, live asset parity, health/readiness, first-screen clarity, four-action sample and reset, invalid/recovery focus, route/title/legal/404 checks, same-origin privacy, security headers, reduced motion, route axe scans, and Lighthouse 100/100/100/100.

M1 is not accepted. Repair both findings, retain the 16 passing claim outcomes, then repeat fresh independent verification and strict review. External dependencies and later M2–M4 scope remain unchanged from the sections above.

## Repair 4 update — 2026-09-06 UTC

### Result

M1 repair 4 is implemented, pushed, deployed, and ready for fresh independent verification and strict review. It is not marked accepted by this handoff.

- Deployed implementation SHA: `4c7ede4443be1866d1b0042925e87c7757394028`
- Supporting repair SHA: `3ec1c927019557ddc83d7524467c48e99a8ea72b`
- Documentation and verification-evidence SHA: `3cce8efefa921a8e1de4eccb4c54d2711fabd2fa`
- Live health: `{"status":"ok","build_sha":"4c7ede4443be1866d1b0042925e87c7757394028"}`
- Live readiness: database and malware scanner both report `ready`.

### Strict-review findings repaired

1. **P0 rotating `car_demo` cookies:** A rate decision now evaluates the stable server-issued visitor identity when present. Direct API callers that only receive rotating demo/client cookies retain a stable anonymous fallback and the first `X-Forwarded-For` identity. A rotating cookie cannot create a fresh write bucket. The regression follows each returned `car_demo` cookie and observes three `201` responses followed by two `429` responses with `Retry-After`. A separate response-outcome test proves independent server-issued visitor cookies retain their own allowance, so isolated browser demos do not throttle each other merely for sharing one ingress IP.
2. **P1 200% phone text sizing:** Removed the scalable root minimum width, allowed mobile header/banner controls to wrap, and permitted staff-grid sheets to shrink in their single-column layout. A Playwright test changes the root text size to 200% at 390 px, proves no horizontal page scroll, and confirms both demo controls remain visible.

### Verification

From the documented dependency setup:

```sh
npm ci
npm run check
npm test
npm run test:e2e
npm run build
```

- `npm run check` passed: 0 Svelte errors/warnings, rustfmt, and clippy with warnings denied.
- `npm test` passed: 5 web tests, 7 Rust unit tests, and 8 Rust integration tests; the Rust coverage includes the rotating-cookie regression.
- `npm run test:e2e` passed: 19/19 local browser tests, including the 200% phone reflow check.
- `npm run build` passed: public JavaScript is 28.91 KiB gzip and CSS is 4.91 KiB gzip.
- Every one of the 16 commands in `.factory/claims.json` was run separately after the final repair; all passed. This includes the exact 5 MiB/one-byte-over upload boundary, controlled 24-hour file expiry, and zero-delivery demo reminder outcome.

Against `https://client-action-room.sociobot.in`:

- The public suite passed with 17 public tests. The controlled-clock file-expiry and local-auth real-workspace tests remain intentionally local-only and passed locally.
- A direct fresh client that preserved each returned `car_demo` cookie received `201, 201, 201, 429, 429`; each limited response carried `Retry-After: 59`.
- Fresh desktop (1280×900) and phone (390×844) contexts showed the job, audience, and “Try it with sample data” before scrolling. The one-click room showed four realistic actions, the persistent sample label, and reset restored the sample. The phone at 200% root text had no horizontal page scroll.
- The supplied URL verifier passed on HTTPS with the expected title, `lang=en`, one H1, one main landmark, alt coverage, named buttons, and no console errors. Playwright axe scans across `/`, `/demo`, `/privacy`, `/terms`, `/workspace`, and the designed 404 route at desktop and phone widths found zero serious or critical issues and no horizontal overflow. The 404 returned its expected HTTP 404 status.
- `@axe-core/cli` was attempted but could not locate a system Chrome binary in this container. The installed Playwright Chromium completed the equivalent route scans above.
- Lighthouse produced 100 performance, accessibility, best-practices, and SEO; LCP was 1.4 s, CLS 0, and transfer weight 99 KiB. It emitted a non-fatal browser-tab-crash message after generating the report.
- Live root headers include CSP with response-header `frame-ancestors 'none'`, HSTS, nosniff, no-referrer, Permissions Policy, and COOP.

### Current milestone and remaining work

- Current milestone: **M1 repair 4**. Fresh independent verification and strict review are the remaining acceptance gates.
- External dependency: the operator must confirm the shared Entra callback registration. The factory must register the recurring Sociobot prices and entitlement contract before M2 billing.
- Later work: M2 membership, export/delete, retention, subscription enforcement, and backup operations; M3 real-workspace authoring for upload, choice, and external-link actions; M3/M4 opted-in transactional reminder delivery. Demo reminders intentionally create no delivery entry.
- The product makes no offline or update promise.

The catalog description remains the 94-character verb-first description in `.factory/catalog-description.txt` and is copied to `/work/.evidence/catalog-description.txt`.

## Verification 5 update — 2026-09-06 UTC

Fresh independent verification **PASSed** M1 repair 4 with zero findings and zero untested declared claims. The full report is `.factory/verification-5.md`.

- Reviewed implementation: `4c7ede4443be1866d1b0042925e87c7757394028`.
- Reviewed documentation pointer: `7fa02cc606ff7e6e15dfe24137c15f1c5e8ee79f`. Its diff from the implementation changes only factory documentation; live `/health` reports this later documentation SHA and live public asset hashes match a fresh candidate build.
- Clean documented gates passed: `npm ci`, `npm test` (5 web, 7 Rust unit, 8 Rust integration), `npm run check`, `npm run build`, and 19/19 local E2E tests.
- Every one of the 16 declared claim commands passed separately. This includes exact 5 MiB acceptance and one-byte rejection, controlled 24-hour upload expiry, and a demo reminder status of one scheduled reminder with zero delivery-queue entries.
- The live public suite passed 17 public tests. The controlled-clock expiry and local identity fixture are intentionally local-only and passed locally.
- Fresh desktop and phone loads showed the job, audience, and sample action before scrolling. The sample had a persistent label and reset restored four realistic actions. At 390 px and 200% text, there was no horizontal scroll and both demo controls remained visible.
- A direct rotating-`car_demo` probe returned `201, 201, 201, 429, 429`; both limited responses carried `Retry-After: 59`.
- Live health and readiness passed. Local integration tests passed tenant isolation, a second app-instance session check, and snapshot/restore persistence. No live restart was initiated because this verifier has no restart authority.
- The supplied URL verifier and Playwright axe scans across key routes at desktop and phone widths passed. `@axe-core/cli` could not locate a system Chrome binary; Playwright Chromium covered the same serious/critical axe check.

M1 now awaits the separate strict review before acceptance. Current external dependencies and later M2–M4 scope are unchanged: Entra redirect confirmation and recurring billing registration; then membership/export/delete/retention/backups, real-workspace non-approval authoring, and opted-in transactional email. Demo reminders intentionally do not deliver email. The product makes no offline or update promise.

## Strict review 2 update — 2026-09-06 UTC

Strict review 2 **PASSed** M1 repair 4 with **0 findings and 0 untested public claims**. The complete report is `.factory/review-2.md`; this accepts M1.

- Reviewed implementation: `4c7ede4443be1866d1b0042925e87c7757394028`.
- Reviewed documentation checkout: `f62225e561615f8aa8a18d54bda34904fa4e25bf`; live health reports the earlier documentation-only pointer `7fa02cc`. Fresh local JS and CSS hashes exactly match the live public assets, so the runtime remains the implementation candidate.
- Clean gates passed after `npm ci`: `npm test` (5 web, 7 Rust unit, 8 Rust integration), `npm run check`, `npm run build`, and 19/19 local E2E tests. All 16 declared claim commands were run separately and passed.
- The three repaired promises are independently recorded: a 5 MiB PDF passes while one extra byte fails; the controlled local clock serves bytes at 23:59:59 and denies them at 24:00:00; one demo reminder reports zero delivery-queue entries.
- The live E2E suite passed 17 public tests. Its two intended local-only fixtures (controlled-clock file expiry and local test identity) passed locally. A rotating-cookie session probe received `201, 201, 201, 429, 429` with `Retry-After: 58` on both limited responses.
- Fresh desktop and phone checks confirmed the job, audience, first sample action, four-action labelled/resettable sample, invalid-form recovery focus, 200% phone-text reflow without horizontal scroll, legal routes, expected designed 404, headers, and same-origin demo traffic. The supplied URL verifier and Playwright axe scans passed. `@axe-core/cli` could not locate a system Chrome binary; Playwright Chromium supplied the serious/critical axe coverage.

M1 acceptance does not include future work. External dependencies remain shared-Entra callback registration and recurring billing registration for M2. M2 membership/export/delete/retention/backups, M3 real-workspace non-approval authoring, and M3/M4 opted-in transactional reminder delivery remain unshipped. The product makes no offline or update promise.
