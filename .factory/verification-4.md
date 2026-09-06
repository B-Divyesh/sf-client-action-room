# Independent verification 4 — Client Action Room

- **Candidate implementation reviewed:** `87e07dcd5a3b95eac9e1a71a1d42456614440757`
- **Documentation / report SHA reviewed:** `b1e96e09fe403c15a2626e92a7603e51ecd1715a`
- **Live URL:** <https://client-action-room.sociobot.in>
- **Verified:** 2026-09-06 UTC
- **Current milestone:** M1 repair 3
- **Verdict:** **PASS — zero findings and zero untested declared claims. M1 may proceed to strict review/acceptance.**

The live `/health` endpoint reported the later documentation SHA `b1e96e0`; `/ready` returned `database: ready` and `malware_scanner: ready`. The assigned implementation candidate is `87e07dc`. The later commits contain documentation, plan, and browser-test wait changes only. A fresh build from this clean checkout matched the live main JavaScript and CSS byte-for-byte, so the deployed client behavior is the reviewed candidate behavior despite the later build identifier.

## First read and demo

Before scrolling, fresh desktop (1280×900) and phone (390×844) browsers both had `scrollY = 0` and showed:

- **Job:** get client actions done on time.
- **Audience:** small firms chasing approvals, files, choices, and payment links across email.
- **First action:** **Try it with sample data**.

Each first action opened the realistic four-action Northline Studio / Alder Street Bakery sample. The persistent banner said **“Demo — sample data, nothing is saved”**; **Reset demo** restored the seed. These checks used new browser contexts and produced no console errors. Sample changes remained inside isolated demo sessions; no real workspace was opened or changed.

## Clean checkout gates

From the clean `b1e96e0` checkout:

```sh
npm ci
npm test
npm run check
npm run build
npm run test:e2e
```

All passed.

- `npm ci`: 89 packages installed; 0 vulnerabilities reported.
- `npm test`: 5 Vitest tests, 6 Rust unit tests, and 8 Rust integration tests passed.
- `npm run check`: Svelte reported 0 errors and 0 warnings; rustfmt and clippy with `-D warnings` passed.
- `npm run build`: built `dist/` and the release server binary. Main public JS is 28.91 KiB gzip and CSS is 4.91 KiB gzip.
- `npm run test:e2e`: 18/18 local browser tests passed.

The public live suite also passed: 16 tests passed, with only the intentionally local-only controlled-clock `file-expiry` test and local-auth `real-workspace` fixture skipped. The local suite executed both of those omitted live fixtures.

## Declared claims

Every exact command in `.factory/claims.json` was run separately from the clean setup. All 16 passed; there are **zero untested claims**.

| Claim ID | Result | Observable evidence |
|---|---|---|
| `demo-one-click` | PASS | Landing action opened the seeded four-action room. |
| `demo-reset` | PASS | Two isolated sessions differed after one completion; reset restored the exact seed. |
| `client-no-account` | PASS | Signed-out client approved through a scoped link without account input. |
| `deadline-order` | PASS | Open actions were ascending by deadline, with the overdue upload first. |
| `approval-audit` | PASS | Approval stored decision, Maya Chen, and server time. |
| `link-expiry` | PASS | Seven-day link lifetime was exact; expired read and direct submit were denied. |
| `secure-upload` | PASS | Non-PDF and EICAR were rejected; clean PDF reached the audit. |
| `upload-5mb-boundary` | PASS | Exactly 5 MiB completed; 5 MiB plus one byte showed the size error. |
| `file-expiry` | PASS | Controlled clock read clean bytes one second before 24 hours, then got `410 demo_expired` at the boundary after purge. |
| `choice-flow` | PASS | The scoped client selected Square pastry crop and it reached the audit. |
| `external-link` | PASS | The client saw `example.com`, received the configured HTTPS URL, and the audit recorded an open. |
| `reminder-audit` | PASS | One reminder recorded the staff actor and server time. |
| `demo-reminders-no-email` | PASS | One schedule produced `scheduled_reminders: 1` and `delivery_queue_entries: 0`; browser traffic stayed same-origin. |
| `real-workspace` | PASS | Local auth fixture created an empty durable firm approval workspace, preserved completion, and denied another owner. |
| `demo-privacy` | PASS | Demo lifetime was at most 24 hours; fragment tokens were not requested; leaving deleted the room; traffic stayed same-origin. |
| `staff-auth` | PASS | Missing and invalid staff tokens returned `401` with `WWW-Authenticate: Bearer`. |

This closes Verification 3’s three P1 gaps: the exact upload boundary, 24-hour uploaded-file availability boundary, and zero-delivery demo reminder behavior all have dedicated tagged tests that exercise the outcomes.

## Live runtime and recovery checks

- `/health` returned 200 and the live SHA above; `/ready` returned healthy database and malware scanner states.
- A fresh visitor made five demo-session requests: `201, 201, 201, 429, 429`. Both `429` responses included `Retry-After: 59`, proving the live allowance is enforced.
- Backend isolation and restart persistence passed in the fresh Rust integration suite (`staff_workspace_is_stable_for_oid_and_isolated_from_another_oid`, `session_survives_a_request_served_by_a_second_app_instance`, and `successful_mutation_can_be_snapshotted_and_restored`). No live restart was initiated by this verification because the work order has no deployment action and the product contract forbids infrastructure changes; the prior deployed restart evidence remains documented in the handoff.
- Invalid client approval submission announced “Choose approve or ask for changes.” and focused `#approval-error`. A change request without a note announced the recovery step and kept that focus. Adding a note then recorded the request successfully.
- The public `upload-5mb-boundary` and `demo-reminders-no-email` paths passed against live. The controlled-clock file-expiry proof is deliberately local-only so production cannot be clock-controlled.

## Accessibility, routes, privacy, and presentation

- The supplied `/opt/fleet/lib/verify-url.sh` passed against live. Its evidence is `.factory/evidence/verification-4-url/verify.json`: title present, `lang=en`, one h1, one main landmark, no image missing alt text, no unnamed buttons, and no console errors.
- Playwright axe found zero serious or critical violations on `/`, `/demo`, `/privacy`, `/terms`, `/workspace`, and `/missing-record` at both desktop and phone widths. There was no horizontal overflow. In reduced-motion mode the observed transition duration was `1e-05s`.
- `npx @axe-core/cli https://client-action-room.sociobot.in --exit` was attempted but its Selenium wrapper could not locate a Chrome binary in this container. This is an environment limitation, not a product finding; the repository’s Playwright axe integration and the independent route scans above used the installed Playwright Chromium and passed.
- All normal routes returned 200 with their route-specific titles. The deliberate unknown-route response returned HTTP 404 with the designed “We could not find this page” screen, one h1, one main landmark, and a working back path. Its browser resource error is expected for the deliberate 404, not a defect.
- The landing, demo, and client flows made only same-origin product requests except when the client explicitly chose the disclosed external HTTPS destination. Fonts, imagery, and scripts are self-hosted. `/privacy`, `/terms`, `robots.txt`, `sitemap.xml`, favicon, internal links, and the labelled Param Factory external link all resolved correctly.
- Live root headers include CSP with response-header `frame-ancestors 'none'`, HSTS, nosniff, no-referrer, COOP, and Permissions Policy. The live client JS SHA is `f09566501f9bd6004f5804e53f30e1e508e13c9c6fabdc7237f4f2a99ecb3814`, matching the fresh build; the CSS SHA also matched exactly.

## Earlier findings and minor checks

| Earlier item | Current disposition | Evidence |
|---|---|---|
| Live rate limit bypass | Fixed | Fresh live strict-session probe returned 429 plus Retry-After after three writes. |
| Demo persistence/restart concern | Fixed | Fresh integration restart/snapshot coverage passed; previous live restart evidence remains documented. |
| Upload scanner / clean-file representation | Fixed | Clean, non-PDF, and EICAR paths passed; readiness reports the scanner ready. |
| Seeded “Start for real” and tenant isolation | Fixed for M1 approval scope | Local real-workspace claim proved empty durable owner isolation and client approval. |
| Preview-only demo action types | Fixed in M1 demo | Upload, choice, external-link, and reminder outcomes passed. |
| First-screen clarity, persistent sample label, reset, phone, keyboard/focus, reduced motion, legal routes, and designed 404 | Passing | Fresh desktop/phone contexts, keyboard/recovery checks, supplied URL verifier, and axe checks passed. |
| Verification 3’s three untested promises | Fixed | Three dedicated claimed outcome commands passed independently. |

## Milestone and external dependencies

This verification accepts the present **M1 repair 3** scope: the account-free demo with all four sample action flows, and the durable signed-in approval workspace loop tested locally. It does not represent future work as shipped.

External dependencies and later milestone work remain separate, not findings against M1:

- Operator confirmation that the shared Entra redirect URI is registered.
- Factory registration of the recurring Sociobot billing prices and entitlement contract for M2.
- M2 membership, export/delete, retention controls, subscription enforcement, and backup operations.
- M3 real-workspace authoring for upload, choice, and external-link actions; transactional reminder delivery and opt-in controls are later work. Demo reminders intentionally create no delivery entry.

