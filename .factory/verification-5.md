# Client action queue verification — PASS

- **Date:** 2026-09-06 UTC
- **Verdict:** **PASS — 0 findings, 0 untested declared claims.**
- **Current milestone:** M1 repair 4
- **Implementation reviewed:** `4c7ede4443be1866d1b0042925e87c7757394028`
- **Documentation pointer reviewed:** `7fa02cc606ff7e6e15dfe24137c15f1c5e8ee79f`
- **Live URL:** <https://client-action-room.sociobot.in>

The live `/health` response reports `7fa02cc606ff7e6e15dfe24137c15f1c5e8ee79f`. That later commit changes only `.factory/handoff.md` and `.factory/plan.md` relative to implementation `4c7ede4`; it is documentation, not a new product image. A fresh build at the documentation pointer produced the same public CSS and JS hashes served live, so the deployed runtime matches the reviewed repair implementation.

## First screen and sample

Fresh desktop (1280 × 900) and phone (390 × 844) browser contexts showed the job, audience, and first action before scrolling:

- Job: **Get client actions done on time**.
- Audience: small firms chasing approvals, files, choices, and payment links across email.
- First action: **Try it with sample data**.

The one-click sample opened the Alder Street Bakery queue with four realistic actions. The persistent label read **“Demo — sample data, nothing is saved.”** Reset restored the four-action seed. A direct `/?demo=1` load also opened the labelled sample room. The declared demo isolation, reset, and no-email tests passed, so no real firm data or real recipients were used.

## Clean checkout gates

A disposable clone at `7fa02cc` was installed with `npm ci` (89 packages; 0 vulnerabilities). These documented commands passed:

| Command | Result |
| --- | --- |
| `npm test` | PASS — 5 web tests, 7 Rust unit tests, and 8 Rust integration tests |
| `npm run check` | PASS — 0 Svelte errors/warnings; rustfmt and clippy with warnings denied passed |
| `npm run build` | PASS — `dist/` and the Rust release binary produced |
| `npm run test:e2e` | PASS — 19/19 local browser tests |

The public initial bundle is 28.91 KiB gzip JavaScript and 4.91 KiB gzip CSS. The lazy authentication chunk is not part of the first load.

## Declared claims

Every command in `.factory/claims.json` was run separately from the clean checkout. All 16 passed; no public promise reviewed here lacked a corresponding declared outcome test.

| Claim | Observable result |
| --- | --- |
| `demo-one-click` | The landing action opened the seeded four-action room. |
| `demo-reset` | Separate sample rooms stayed isolated; reset restored the exact seed. |
| `client-no-account` | A signed-out client recorded an approval without account input. |
| `deadline-order` | Open requests were ordered by deadline, with the overdue upload first. |
| `approval-audit` | The audit recorded decision, Maya Chen, and server time. |
| `link-expiry` | A seven-day link expired exactly and denied read and submission. |
| `secure-upload` | Invalid type and EICAR were rejected; a clean PDF reached the audit. |
| `upload-5mb-boundary` | Exactly 5 MiB completed; 5 MiB plus one byte showed the visible size error. |
| `file-expiry` | Bytes were readable at 23:59:59 and returned `410 demo_expired` at 24:00:00 after purge. |
| `choice-flow` | The scoped client choice reached the audit. |
| `external-link` | The HTTPS destination was disclosed and the audit recorded an open, not a payment. |
| `reminder-audit` | One scheduled reminder recorded the staff actor and server time. |
| `demo-reminders-no-email` | One schedule reported one reminder and zero delivery-queue entries; traffic was same-origin. |
| `real-workspace` | The local identity fixture created an empty durable approval workspace and denied another owner. |
| `demo-privacy` | Fragment tokens were not requested, demo lifetime was at most 24 hours, and leaving removed the room. |
| `staff-auth` | Missing and invalid staff tokens both returned `401` and `WWW-Authenticate: Bearer`. |

## Live checks

`PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e` passed. It ran 17 public tests. The controlled-clock file-expiry test and the local test-identity real-workspace fixture were intentionally skipped live and both passed locally.

- `/health` returned `ok`; `/ready` returned `database: ready` and `malware_scanner: ready`.
- A direct API client preserved each returned rotating `car_demo` cookie and received `201, 201, 201, 429, 429`. Both limited responses included `Retry-After: 59`.
- The live 390 px demo at 200% root text had no horizontal page scroll. `Reset demo` and `Start for real` remained visible.
- Local integration coverage passed for organization isolation, a second application instance serving the same session, and successful mutation snapshot/restore. A live restart was not initiated because this verification has no deployment or restart authority.
- Internal routes `/`, `/demo`, `/privacy`, `/terms`, and `/workspace` returned 200. The styled missing route returned its intentional 404 and offered a return path. The lone 404 console message was the browser reporting that deliberate HTTP 404, not a broken page.
- Route link checks passed for navigable internal URLs, the Param Factory external link, and the two explicit `mailto:` contacts. The skip link on the already-missing route resolves to that same intentional 404 document.
- The live response has CSP with response-header `frame-ancestors 'none'`, HSTS, `nosniff`, no-referrer policy, permissions policy, and COOP. Demo browser requests stayed same-origin except for a client-selected external destination.

## Accessibility and responsive checks

The factory URL verifier passed with HTTPS 200, title, `lang=en`, one H1, one main landmark, image alt coverage, named buttons, and no normal-load console errors.

Playwright axe scans on `/`, `/demo`, `/privacy`, `/terms`, `/workspace`, and the styled missing route at desktop and 390 px widths found zero serious or critical violations. Each route had one H1, one main landmark, correct route title, and no horizontal overflow. Keyboard skip-link, route-heading focus, error announcement/recovery, and reduced-motion behavior passed in the public suite.

`npx @axe-core/cli` was attempted but could not locate a system Chrome binary in this worker. This is an environment limitation, not a product finding: the installed Playwright Chromium completed the route-and-width axe coverage above.

## Earlier findings and minor checks

| Earlier item | Current disposition |
| --- | --- |
| Live demo state loss | Fixed — live public demo/client flows passed; local second-instance and snapshot/restore integration tests passed. |
| Missing current-M1 action paths | Fixed for the stated M1 scope — sample upload, choice, external-link, and reminder paths passed; real approval workspaces passed locally. |
| Missing or bypassable rate limits | Fixed — rotating-cookie live probe was limited after three creates with `Retry-After`. |
| Upload byte heuristic | Fixed — scanner readiness, clean PDF, invalid type, and EICAR outcomes passed. |
| Untested 5 MiB, 24-hour expiry, and no-email promises | Fixed — all three have dedicated passing claim tests. |
| 200% text caused phone overflow | Fixed — fresh live 390 px test reflowed with both demo controls visible. |
| First-screen clarity, sample label/reset, keyboard/focus, reduced motion, legal pages, privacy traffic, and designed 404 | Passing. |

## Scope and external dependencies

This PASS covers M1 repair 4 only. It does not claim future work early.

- The operator still needs to confirm the shared Entra callback registration for `https://client-action-room.sociobot.in/auth/callback`.
- Recurring Sociobot billing registration and entitlement work belong to M2.
- Membership, export/delete, retention, and backup operations are M2 work.
- Real-workspace upload, choice, and external-link authoring are M3 work. Opted-in transactional reminder delivery is M3/M4 work; demo reminders intentionally create no delivery entry.
- The product makes no offline or update promise.

Docker is not installed in this disposable worker, so no local container smoke was possible. The documented native release build passed and the deployed service was exercised live.

M1 has passed this fresh independent verification. A strict review remains the separate acceptance gate before M1 is accepted.
