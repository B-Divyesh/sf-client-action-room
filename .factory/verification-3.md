# Independent verification 3 — Client Action Room

- **Candidate implementation:** `b059adf1d4b08f755a2d1a08d3a77e2052586648`
- **Documentation / report commit reviewed:** `d4dd83acf0fea73c3da8b04274b4cf64d35f9547`
- **Live URL:** <https://client-action-room.sociobot.in>
- **Verified:** 2026-09-06 UTC
- **Current milestone:** M1 repair 2
- **Verdict:** **FAIL — one P1 finding; do not mark M1 accepted yet.**

The live health endpoint reports `d4dd83acf0fea73c3da8b04274b4cf64d35f9547`. The implementation paths are unchanged between the stated candidate and that documentation commit. A fresh build from the clean checkout exactly matched the live main JS and CSS bytes, so the live product is the reviewed candidate implementation with the later documentation build identity.

## First read

Before scrolling, a fresh desktop browser (1280×900) and phone browser (390×844) both showed the same complete first action:

- **Job:** get client actions done on time.
- **Audience:** small firms chasing approvals, files, choices, and payment links across email.
- **First action:** **Try it with sample data**; it opens a ready room in one click and says that nothing is saved to the visitor account.

There was no horizontal overflow or console error in either cold load.

## Finding

### P1 — three public promises have no observable claim test

The 13 declared claims all have tagged tests and every declared command passed. However, the public UI and README make these additional, concrete promises without a corresponding `.factory/claims.json` entry and observable sandbox assertion:

1. The upload form says **“PDF, up to 5 MB”** (`src/App.svelte`). `secure-upload` checks a text file, EICAR, and a small clean PDF, but never exercises the 5 MB boundary.
2. The upload form says **“Files expire within 24 hours.”** No declared test asserts the stored file expiry or that expired bytes are unavailable/deleted.
3. The landing page says the demo **“records reminder schedules but does not send email.”** `reminder-audit` proves that a schedule is recorded, but no test proves the non-delivery promise.

These are user-reliant product claims. Under the claims contract they must each be listed and tested from the demo, or the copy must be removed or narrowed. This is one P1 claim-coverage finding containing **three untested public claims**.

## What passed

### Clean checkout and declared claims

A fresh clone at `d4dd83a` was installed with `npm ci` (89 packages, 0 vulnerabilities). These commands passed:

```sh
npm test
npm run check
npm run build
npm run test:e2e
```

- `npm test`: 5 Vitest tests, 6 Rust unit tests, and 8 Rust integration tests passed.
- `npm run check`: Svelte check reported 0 errors/warnings; rustfmt and clippy passed.
- `npm run build`: produced `dist/` and the release server binary. Main JS is 28.90 KiB gzip; CSS is 4.91 KiB gzip.
- Local browser suite: 15/15 passed, including the real-workspace isolation fixture.

Every exact command in `.factory/claims.json` was run separately from that clean checkout. All 13 passed: `demo-one-click`, `demo-reset`, `client-no-account`, `deadline-order`, `approval-audit`, `link-expiry`, `secure-upload`, `choice-flow`, `external-link`, `reminder-audit`, `real-workspace`, `demo-privacy`, and `staff-auth`.

Against the live URL, the same browser suite passed all 14 public tests; `real-workspace` was intentionally skipped because its local `AUTH_TEST_MODE` fixture is disabled in production. This skip is not an untested declared claim because the local claim command passed; it is a production-auth-fixture limitation.

### Demo, invalid paths, and recovery

The one-click sample opened an isolated Northline Studio / Alder Street Bakery room with four realistic actions. The persistent banner read **“Demo — sample data, nothing is saved”** and offered **Reset demo** and **Start for real**. Reset restored the seed; an approval was completed without account navigation and appeared in the dated staff audit. Browser request recording found only same-origin demo traffic.

Invalid and recovery checks passed live: submitting an approval without a decision announced “Choose approve or ask for changes.” and moved focus to `#approval-error`; requesting changes without a note announced the specific recovery message; entering the note successfully recorded the change request. Expired links disclosed no request/workspace details. The safe upload, EICAR rejection, choice, external-destination disclosure, and reminder audit paths passed in the browser suite.

### Backend boundaries and persistence

- `/health` returned 200; `/ready` returned `database: ready` and `malware_scanner: ready`.
- With one fresh browser visitor identity, five demo-session requests returned `201, 201, 201, 429, 429`; both 429 responses included `Retry-After: 59`.
- A demo room recorded a reminder, then the active product revision was restarted. After it became healthy and returned to one running replica, the same cookie state reloaded the same dated reminder audit entry. This independently verifies restart persistence.
- The local real-workspace claim created a new empty firm workspace, completed an approval through a signed-out client link, retained the result, and denied a distinct owner. The live API rejected missing and invalid staff tokens with `401` and `WWW-Authenticate: Bearer`.

### Accessibility, privacy, routes, and presentation

- The worker URL verifier passed: HTTPS 200, no console errors, title, `lang=en`, one h1, one main landmark, image alt coverage, and named buttons.
- `npx @axe-core/cli` was attempted but cannot locate Chrome in this container. Playwright axe integration was used instead: it found no violations at any severity on `/`, `/demo`, `/privacy`, `/terms`, `/workspace`, or the designed 404 route at both desktop and phone widths.
- Keyboard skip-link, h1 focus after navigation, focus rings, mobile layout, legal pages, per-route titles, same-origin demo traffic, and the designed HTTP 404 all passed. The deliberate 404 status is expected and is not a defect.
- Under reduced motion, measured transition/animation durations were `0.00001s`; no horizontal overflow appeared at 390 px. The product makes no offline or update promise.
- All internal navigation routes returned 200 (`/`, `/demo`, `/privacy`, `/terms`, `/workspace`, and the in-page how-it-works anchor). The intentional missing route returned the designed 404.
- Live headers included CSP, HSTS, `X-Content-Type-Options`, `Referrer-Policy`, COOP, Permissions Policy, and immutable caching for hashed assets.

## Earlier findings: current disposition

| Earlier finding | Disposition | Independent evidence |
|---|---|---|
| Live rate limiting was bypassable | Fixed | strict live session allowance returned 429 plus `Retry-After` after three writes. |
| Demo state was lost between instances/restart | Fixed | same demo reminder record remained after a live revision restart. |
| Upload scan was only a byte check | Fixed | live `/ready` reports the malware scanner ready; browser secure-upload flow passed EICAR rejection and clean completion. |
| “Start for real” used seeded demo data / no tenant isolation | Fixed for M1’s approval scope | local real-workspace claim proved empty durable workspace, client completion, and other-owner denial. |
| Preview-only non-approval actions | Fixed in the demo | upload, choice, external link, and reminder flows passed. |
| First-screen copy, demo label/reset, mobile, keyboard, focus, reduced motion, titles, legal pages, 404, same-origin traffic | Fixed / passing | fresh desktop and phone checks plus the browser and axe runs above. |
| Earlier unlisted public claims | Improved, but not fully fixed | the declared list now has 13 tests; the three untested claims in the P1 finding remain. |

## Current milestone and external dependencies

This is M1 repair 2. The M1 demo and the real approval workspace loop are in scope and were exercised. The following are future work or external dependencies, not defects against the current milestone:

- Operator confirmation/registration of the shared Entra redirect URI.
- Factory registration of recurring Sociobot billing prices and entitlement contract for M2.
- M2 membership, export/delete, retention controls, subscription enforcement, and backup operations.
- M3 real-workspace authoring for upload, choice, and external-link actions; reminder email delivery is later work and is not advertised as delivered for real workspaces.

## Required repair

Add three separately tagged demo claim tests (or remove/narrow the three statements): exact 5 MB acceptance/rejection boundary, 24-hour file expiry/deletion, and no email delivery for demo reminder scheduling. Re-run all claim commands and this independent verification after the repair.
