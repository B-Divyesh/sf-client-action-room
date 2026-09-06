# Strict review 2 — Client Action Room

- **Verdict:** **PASS — 0 findings, 0 untested public claims. M1 is accepted.**
- **Current milestone:** M1 repair 4
- **Candidate implementation reviewed:** `4c7ede4443be1866d1b0042925e87c7757394028`
- **Documentation checkout reviewed:** `f62225e561615f8aa8a18d54bda34904fa4e25bf`
- **Live health build identifier:** `7fa02cc606ff7e6e15dfe24137c15f1c5e8ee79f`
- **Live URL:** <https://client-action-room.sociobot.in>
- **Reviewed:** 2026-09-06 UTC

The candidate implementation and the documentation checkout differ only in
factory reports, plan status, and handoff text. The live health identifier is
also a later documentation pointer. A fresh build from this checkout served
the same public assets as the live product: JavaScript SHA-256
`4d61e183ad87d61b5b105e48ce0f0cadbd2299a8635cd5ecc36fbc5030f5b273`
and CSS SHA-256
`9acd74c1dcb265f9440159d4a1894b9b5c5c27427340bbf3e6f0ecc36e29204a`.
The deployed runtime therefore matches implementation `4c7ede4`.

## Job, audience, and first action

Fresh desktop (1280 × 900) and phone (390 × 844) contexts showed all of the
following before scrolling:

- **Job:** Get client actions done on time.
- **Audience:** Small firms chasing approvals, files, choices, and payment
  links across email.
- **First action:** Try it with sample data.

The first action opened the Alder Street Bakery sample room. It contained four
realistic requests and the persistent label **“Demo — sample data, nothing is
saved.”** Reset restored the seed. The checks used new demo contexts only; no
firm workspace or real recipient was used.

## Clean checkout gates

After `npm ci` (89 packages; 0 vulnerabilities), the documented commands
passed:

| Command | Result |
|---|---|
| `npm test` | PASS — 5 web tests, 7 Rust unit tests, 8 Rust integration tests |
| `npm run check` | PASS — 0 Svelte errors/warnings; rustfmt and clippy with warnings denied |
| `npm run build` | PASS — produced `dist/` and the release server |
| `npm run test:e2e` | PASS — 19/19 local browser tests |

The public initial JavaScript is 28.91 KiB gzip and CSS is 4.91 KiB gzip.

## Declared claims

Every exact command declared in `.factory/claims.json` was run separately from
the clean checkout. All 16 passed. The public landing page, demo, client
screens, privacy page, terms, and README were cross-checked against the claim
list. No additional user-reliant current-M1 promise was found without an
observable sandbox outcome.

| Claim | Result exercised |
|---|---|
| `demo-one-click` | One action opened the seeded four-action room. |
| `demo-reset` | Separate samples stayed isolated and reset restored the seed. |
| `client-no-account` | A client approved without account input. |
| `deadline-order` | Open requests were sorted by deadline. |
| `approval-audit` | Decision, actor label, and server time were recorded. |
| `link-expiry` | A seven-day link then denied reading and submission. |
| `secure-upload` | Type rejection, EICAR rejection, clean scan, and scope denial passed. |
| `upload-5mb-boundary` | Exactly 5 MiB completed; 5 MiB plus one byte showed the size error. |
| `file-expiry` | Bytes read at 23:59:59; access returned `410 demo_expired` at 24:00:00. |
| `choice-flow` | The selected scoped option reached the audit record. |
| `external-link` | The HTTPS destination was disclosed and only the open was recorded. |
| `reminder-audit` | One reminder and its staff audit entry were recorded. |
| `demo-reminders-no-email` | One schedule reported zero delivery-queue entries and same-origin traffic. |
| `real-workspace` | An empty durable approval workspace persisted and another owner was denied. |
| `demo-privacy` | Fragment secrecy, same-origin traffic, 24-hour maximum, and leave deletion passed. |
| `staff-auth` | Missing and invalid tokens returned `401` and `WWW-Authenticate: Bearer`. |

The three formerly untested boundary promises are now each independently
exercised by the exact commands below:

```sh
npm run test:e2e -- --grep @claim:upload-5mb-boundary
npm run test:e2e -- --grep @claim:file-expiry
npm run test:e2e -- --grep @claim:demo-reminders-no-email
```

All three passed. The file-expiry proof uses the local controlled clock by
design; production exposes no clock-control endpoint.

## Live runtime and recovery checks

- `/health` returned 200 and `/ready` returned `database: ready` and
  `malware_scanner: ready`.
- `PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e`
  passed 17 live tests. The controlled-clock `file-expiry` case and local
  `AUTH_TEST_MODE` real-workspace fixture were intentionally skipped live and
  passed locally above.
- A fresh direct client retained each returned rotating `car_demo` cookie and
  received `201, 201, 201, 429, 429`; both limited responses included
  `Retry-After: 58`.
- A blank client approval announced **“Choose approve or ask for changes.”**,
  focused `#approval-error`, and successfully completed after choosing an
  approval.
- Local integration tests passed tenant isolation, a second application
  instance serving the same session, and successful mutation snapshot/restore.
  This review did not restart the live service because no restart authority is
  part of this work order.

## Accessibility, privacy, routes, and presentation

- `/opt/fleet/lib/verify-url.sh` passed against the live URL: HTTPS 200,
  title, `lang=en`, one H1, one main landmark, complete image alt coverage,
  named buttons, and no normal-load console errors.
- Playwright axe scans on `/`, `/demo`, `/privacy`, `/terms`, `/workspace`,
  and the styled missing route at phone width found zero serious or critical
  violations. Every route had one H1, one main landmark, and no horizontal
  overflow.
- At 390 px and 200% root text, the demo had no horizontal page scroll; both
  Reset demo and Start for real remained visible.
- The live route titles, keyboard skip link, heading focus, reduced-motion
  behavior, legal routes, same-origin demo traffic, and security headers
  passed. The missing route returned the intended styled HTTP 404 with a path
  home; that status is expected rather than a defect.
- The root response includes CSP with response-header `frame-ancestors 'none'`,
  HSTS, `nosniff`, no-referrer policy, COOP, and Permissions Policy.
- `npx @axe-core/cli` was attempted but its Selenium wrapper could not locate a
  system Chrome binary in this worker. This is an environment limitation, not
  an untested product claim: the installed Playwright Chromium completed the
  route axe checks above.

## Earlier findings and disposition

| Earlier item | Current disposition |
|---|---|
| Live demo state loss | Fixed — public flows pass; second-instance and snapshot/restore integration tests pass. |
| Incomplete current-M1 demo paths | Fixed for the stated milestone — upload, choice, external-link, reminder, and real approval paths pass. |
| Rate-limit absence and rotating-cookie bypass | Fixed — the rotating-cookie live probe reaches 429 with Retry-After after three creates. |
| Seeded real workspace / owner isolation | Fixed for M1 approval scope — local empty-workspace and other-owner denial claim passes. |
| Upload byte heuristic | Fixed — readiness, EICAR rejection, clean scan, and fail-closed scanner tests pass. |
| Untested 5 MiB, 24-hour, and no-email promises | Fixed — three dedicated outcome commands pass. |
| 200% phone text overflow | Fixed — fresh phone reflow has no horizontal scroll and retains both controls. |
| First-screen clarity, demo label/reset, keyboard/focus, reduced motion, legal pages, privacy traffic, and 404 | Passing. |

## Current milestone and external dependencies

This review accepts **M1** only. It does not treat planned work as shipped.

- **External operator dependency:** confirm registration of
  `https://client-action-room.sociobot.in/auth/callback` on the shared Entra
  application.
- **External factory dependency:** register recurring Sociobot billing prices
  and entitlement endpoints before billing can be claimed in M2.
- **Later product work:** M2 membership, export/delete, retention,
  subscription enforcement, and backups; M3 real-workspace upload, choice,
  and external-link authoring; M3/M4 opted-in transactional reminder delivery.
  Demo reminders intentionally create no delivery entry.
- The product makes no offline or update promise.
