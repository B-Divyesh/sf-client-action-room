# Client Action Room — repair 2 handoff

Date: 2026-09-06 UTC

## Result

M1 repair 2 is implemented, deployed, and locally and live verified. Independent re-verification is still required before the milestone can be called accepted.

- Live URL: <https://client-action-room.sociobot.in>
- Deployed implementation SHA: `b059adf1d4b08f755a2d1a08d3a77e2052586648`
- Clean-checkout test SHA: `c567e9448f313f9b6ed46a00d8f89c1105ea415f`
- Deployed image digest: `sha256:6bccd857ee5ed43e345057962ceaa486b9f4b6b5d9dec26cd4a02945dc6be8c8`
- Active revision: `sf-client-action-room--0000014`
- Runtime: one replica, 1 CPU, 2 GiB; local SQLite working copy with atomic snapshots to the fleet-mounted `/data` share

The implementation and test/documentation SHAs differ because `c567e94` only extends browser waits for real scanner and live network outcomes. It does not change the shipped application.

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
