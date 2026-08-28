# Planner handoff

- Work order: `venture-client-action-room-plan`
- Date: 2026-08-28
- Branch: `main`
- Base: `13f88dfd98ce4c3bcc19eb24dde640416cebc9cc`

## What was done

- Wrote `.factory/plan.md` as the venture delivery contract: PRD, three jobs, evidence and wedge, pricing, architecture, tenancy/data model, Entra CIAM, scoped client grants, recurring Sociobot billing boundary, background jobs, uploads/scanning, rate limits, observability, backup/export/delete, design rules, M1–M5 routes/claims/tests/DoD, and risk experiments.
- Wrote `.factory/design.md` for the distinct **municipal archive window** direction. It specifies both themes, type, spacing, shape, motion, responsive and accessible behavior, key screens, asset plan, and provenance requirements.
- Added `.factory/claims.json` with six planned M1 claims and their exact one-test-per-claim Playwright commands/sandboxes.
- Added `.factory/demo.md` with the Northline Studio / Alder Street Bakery seed, 24-hour isolated namespace, reset/exit behavior, and strict no-production-data boundary.
- Added `.factory/component-inventory.md` and a typed 18-component mirror in `src/lib/components/inventory.ts`.
- Updated the brief state to `ADMITTED` with the supplied admission timestamp.
- Scaffolded Svelte 5 + strict TypeScript + Vite, Vitest, pinned Playwright 1.58.2, Rust 2021 + axum, lockfiles, environment example, a non-root multi-stage Dockerfile, and GitHub Actions for checks/tests/build.
- Added only a planning placeholder and `/health`; no client workflow, demo, account, billing, upload, email, or AI feature was built.
- Updated README and ignore rules. No external font, script, image, analytics, or customer data path was added.

## Decisions builders should preserve

- The primary object is an accountable external `action`, not a project/task card.
- M1 proves approval end to end in a fully isolated, resettable anonymous demo. Other sample action types remain clearly preview-only until M3.
- Staff identity uses the shared Sociobot Entra CIAM tenant. External low-risk clients use fragment-carried, hashed, expiring scoped grants exchanged for short-lived HttpOnly cookies.
- Starter is $49/month for 5 active workspaces/3 staff/10 GB. Studio is $99/month for 20 active workspaces/10 staff/50 GB.
- The supplied paid-unlock contract is one-time, while the brief requires subscriptions. M2 must integrate a factory-registered recurring Sociobot entitlement flow and must not label a one-time license as a subscription.
- AI does not earn a core-product place yet. A narrowly defined M5 drafting experiment has an evidence threshold and must use only the Sociobot gateway if built.
- Product art is deferred to M1. The planning scaffold contains no unreviewed/generated imagery or font binary.

## Verification

Completed locally:

- `npm ci`
- `npm run check` — Svelte/TypeScript: 0 errors, 0 warnings; Rust fmt and clippy pass.
- `npm test` — 2 web scaffold tests and 1 Rust health test pass.
- `npm run build` — `dist/` produced; release API produced.
- `npm audit --audit-level=moderate` — 0 vulnerabilities.
- Browser smoke at 390×844 — correct title, exactly one `<h1>`, exactly one `<main>`, no console/page errors.
- API startup with only `PORT=18080` — `/health` returned `{"status":"ok","build_sha":"dev"}`.
- Placeholder build: JS 25.42 KB / 10.12 KB gzip; CSS 2.98 KB / 1.26 KB gzip; HTML 0.62 KB.
- Token spot-check: light text/paper 12.95:1, muted/paper 5.63:1, focus/paper 4.74:1; dark text/canvas 14.91:1, muted/canvas 9.57:1, focus/canvas 9.54:1.

Docker could not be built in this worker because the `docker` command is unavailable. The Dockerfile still follows the supplied build-arg, non-root, `PORT`, and no-`.git` contract; M1 must run the actual container smoke.

The M1 Playwright claim commands are intentionally not runnable yet because this is a planning work order and the product/demo was expressly not built. M1 must implement exactly one tagged test per entry before changing claim status or publishing the copy.

## Needs operator action

- Register `https://client-action-room.sociobot.in/auth/callback` on SPA client `25c704f4-465a-47af-80ab-2c489466b697` before M2 production sign-in.
- Register Starter ($49/month) and Studio ($99/month) as recurring Sociobot/Dodo prices in pilot and production. Provide the recurring checkout, return, webhook, verification, cancellation, and fixture contract to M2. Do not substitute the documented one-time license flow.
- Provision region-specific PostgreSQL, private object storage, ClamAV, and transactional email only in the milestones that use them. The factory owns deployment, DNS, secrets, and billing registration.

## Known gaps and next step

Known gaps are intentional: the placeholder does not serve a real landing page, `/demo`, `/privacy`, `/terms`, static metadata assets, the web shell through axum, or M1 claim tests. The current container copies `dist/` but the health-only API does not serve it yet.

Next work order: **M1 — Public site and approval demo**. Read `.factory/plan.md`, `.factory/design.md`, `.factory/demo.md`, `.factory/claims.json`, and this handoff. Replace the placeholder; do not weaken the demo boundary or convert preview-only M3 action types into claims.
