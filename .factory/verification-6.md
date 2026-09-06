# Independent verification 6 — Client Action Room

- **Verdict:** **PASS — 0 findings, 0 untested claims.**
- **Verified:** 2026-09-06 UTC
- **Current milestone:** M2 builder candidate (not accepted; strict review remains required)
- **Implementation reviewed:** `d82ccfc2d8f27338232192c763e17abcf699d812`
- **Documentation pointer reviewed:** `d83b00c9fb9cf0f52069f72eb1be4da259e79378`
- **Live URL:** <https://client-action-room.sociobot.in>

## Candidate classification

`d83b00c` is a later documentation pointer. Its diff from `d82ccfc` contains only `.factory/handoff-m2.md`, `.factory/handoff.md`, and `.factory/plan.md`; it changes no runtime source, assets, migrations, or dependencies. The live `/health` response currently identifies `d83b00c`, so the deployed build identity is the documentation pointer while the reviewed product implementation remains `d82ccfc`.

## First screen and demo

Fresh desktop (1280 x 900) and phone (390 x 844) Chromium contexts were opened before scrolling.

- **Job:** Get client actions done on time.
- **Audience:** Small firms chasing approvals, files, choices, and payment links across email.
- **First action:** **Try it with sample data**; the adjacent text says that a ready sample room opens and nothing is saved to the visitor account.

Both views showed the complete first action without console errors or horizontal scrolling. The phone view kept the action and three plain facts within the viewport. One click opened the realistic Alder Street Bakery queue with four actions. The persistent label reads **“Demo — sample data, nothing is saved”**; **Reset demo** restored the seed and **Start for real** removed the sample boundary. The sample did not read or write a real firm.

## Clean checkout and claims

A disposable clone at `d83b00c` was installed with `npm ci` (89 packages, 0 vulnerabilities). The following passed:

| Command | Result |
| --- | --- |
| `npm test` | PASS — 5 web tests; 9 Rust unit and 10 Rust integration tests; doc tests passed. |
| `npm run check` | PASS — 0 Svelte errors/warnings; formatting and Clippy with warnings denied passed. |
| `npm run build` | PASS — `dist/` and the locked release binary built. Public JS was 33.28 KiB gzip; CSS 5.15 KiB gzip. |
| `npm run test:e2e` | PASS — 21/21 local browser cases. |

Every exact command in `.factory/claims.json` was then invoked separately from that clone. All 18 returned exit code 0: `demo-one-click`, `demo-reset`, `client-no-account`, `deadline-order`, `approval-audit`, `link-expiry`, `secure-upload`, `upload-5mb-boundary`, `file-expiry`, `choice-flow`, `external-link`, `reminder-audit`, `demo-reminders-no-email`, `real-workspace`, `demo-privacy`, `staff-auth`, `firm-owner-controls`, and `billing-registration-state`.

The controller-requested boundary evidence was outcome-based:

| Public promise | Command | Observed result |
| --- | --- | --- |
| 5 MiB upload boundary | `npm run test:e2e -- --grep @claim:upload-5mb-boundary` | An exact 5 MiB PDF completed scanning; the same otherwise-valid PDF plus one byte showed the visible 5 MB recovery error. |
| 24-hour file expiry | `npm run test:e2e -- --grep @claim:file-expiry` | File bytes were available at 23:59:59; the controlled clock at exactly 24:00:00 purged the record and access returned `410` / `demo_expired`. |
| Demo reminders do not deliver email | `npm run test:e2e -- --grep @claim:demo-reminders-no-email` | Scheduling produced one reminder audit item and the observable status remained zero delivery-queue entries; recorded browser traffic was same-origin. |

No reviewed public promise lacked a listed, observable claim test. The separate command log is retained in the verification worker at `/tmp/client-action-room-verify-6-claims.log`.

## Live behavior

`/health` returned 200 and build id `d83b00c`; `/ready` returned `database: ready` and `malware_scanner: ready`.

`PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e` passed on the final clean run: 18 live cases passed and the three deliberate local-only fixtures (controlled clock, local signed test identity, owner controls) were skipped live and had already passed locally. This covered demo actions, invalid approval recovery, expiry, scanner rejection, privacy requests, titles, legal routes, designed 404, keyboard navigation, focus, reduced motion, Playwright Axe, and 200% text at the 390 px viewport. At 200% text both demo controls remained visible and the page had no horizontal scroll.

One earlier live aggregate attempt received a `503` after a demo-room creation response had already supplied seed data; the UI displayed its documented retry state. An immediate new fresh session returned `201`, the isolated reminder claim passed, and a complete subsequent live suite passed. This was recorded as a recovery observation, not a reproducible finding.

For request allowance, a new browser-issued `car_visitor` cookie was preserved alongside rotating `car_demo` cookies. Five `POST /api/v1/demo/sessions` requests returned `201, 201, 201, 429, 429`; both limited responses supplied `Retry-After: 58`. This confirms a rotated demo cookie cannot reset the stable visitor allowance.

The live response includes CSP with response-header `frame-ancestors 'none'`, HSTS, `nosniff`, no-referrer policy, COOP, and a restrictive Permissions Policy. `npx @axe-core/cli` was attempted but could not launch because this worker has no system Chrome binary. The installed Playwright Chromium Axe integration ran in the passing local and live browser coverage and found no serious or critical violations.

## Earlier findings

| Earlier item | Current disposition |
| --- | --- |
| Rotating cookie request-limit bypass | Fixed — live rotating-cookie probe reached 429 after three creates with `Retry-After`. |
| 200% phone text lost demo controls or scrolled horizontally | Fixed — local and live 390 px regression passed with both controls visible. |
| Unproved 5 MiB, 24-hour file expiry, and no-email statements | Fixed — each has its own passing, observable claim command. |
| Demo isolation, reset, persistence, tenant scope, scanner, legal routes, accessibility, and designed 404 | Passing — retained local and live claim/smoke coverage passed. |

## Current M2 scope and external dependencies

This PASS verifies the M2 candidate’s implemented scope: tenant-scoped firm persistence, roles and owner controls, export/delete recovery, CIAM token boundary, planned-price disclosure, and the retained M1 demo. It does not advance M3 or mark M2 accepted.

- A real hosted Sociobot CIAM sign-in/callback round trip still needs operator confirmation of the registered redirect URI.
- Sociobot must register the recurring Starter ($49/month) and Studio ($99/month) offers and provide the recurring entitlement contract. Checkout is honestly unavailable and no checkout link is exposed.
- Transactional email/outbox delivery belongs to M4. Demo reminders intentionally produce no email or real delivery entry.
- Signed-in authoring for upload, choice, and external-link requests remains M3 scope, not a current M2 shipped claim.

M2 needs a separate strict review with zero findings before acceptance.
