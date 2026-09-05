# Client Action Room venture plan

- Status: **M1 repair 2 complete in source; deployment verification pending**
- Product: `client-action-room`
- Artifact: web application with backend
- Production URL: `https://client-action-room.sociobot.in`
- Plan owner: Param Factory
- Last updated: 2026-08-28

This document is the delivery contract. A milestone may be marked complete only when its routes, tests, claims, and definition of done all pass. Builders must read this plan, `.factory/design.md`, and every earlier milestone handoff before changing code.

## 1. Product requirements

### Customer and situation

The first customer is a 2–20 person agency or professional-service firm. One staff member owns a client relationship and regularly needs a client to approve a draft, upload a file, choose an option, visit a payment or booking link, or answer a short request before work can continue.

Today, that staff member sends email and chat reminders, shares Drive or Notion links, and checks an invoice or project tracker separately. The client sees several tools or a broad portal. Neither side has one short, accountable list of what the client must do next.

The economic buyer is the agency owner or operations lead. Staff create requests. External clients complete them. Clients do not need an account for a low-risk request.

### Promise

**Give each client one short, deadline-based action list.**

For the first screen, use this plain-words copy unless research produces a clearer tested version:

- Headline: **Get client actions done on time**
- Supporting sentence: **For small firms chasing approvals, files, choices, and payment links across email.**
- Primary action: **Try it with sample data**
- Adjacent explanation: **A sample room opens. Nothing is saved to your account.**
- Facts: **Clients do not need an account.** / **Links expire.** / **Plans start at $49 a month.**

The first two facts become public claims only when their M1 tests pass. Pricing may appear when M2 billing is live. M1 must extract all landing copy to `.factory/copy-audit.md`, count words, and remove banned or over-22-word sentences.

### Three jobs the product must nail

1. **Ask clearly.** Staff create a request with one owner, one action, a deadline, and a client-safe link. The page shows only information needed to act.
2. **Finish without friction.** A client opens the link without creating an account, understands the request on a phone, and approves, uploads, chooses, or follows the named external link.
3. **Know what happened.** Staff see what is open, due soon, overdue, and complete. They can send a restrained reminder and retain an exportable audit record.

### Core loop and state model

The primary object is an `action`, not a project or task card.

`draft → open → due_soon → overdue → completed`

An open action can also become `cancelled` or `expired`. Reopening creates a new action version and a new audit event; it does not rewrite the completed record. A client submission is idempotent. Concurrent second submissions receive the recorded result and cannot silently overwrite it.

The smallest complete loop is:

1. Staff create an approval action and issue an expiring client link.
2. Client opens the focused action page and approves or asks for changes.
3. Staff queue updates and the append-only audit ledger records the decision, actor label, link identity, and server time.

M1 proves this loop with an isolated demo. M2 makes it persistent for signed-in firms.

### Success and guardrail measures

These are product targets, not marketing claims until measured in opted-in pilots:

- Primary: at least 60% of requested client actions complete within 72 hours.
- Workflow: participating firms report at least 40% fewer follow-up emails per action against their two-week baseline.
- Activation: a new owner creates a real workspace, publishes an action, and copies its client link within five minutes.
- Client usability: at least 85% of moderated client sessions complete the requested action without help.
- Reliability: 99.9% monthly availability for the action read/submit path; p95 API reads under 300 ms inside the deployment region, excluding file transfer.
- Safety guardrail: zero cross-tenant reads in automated isolation tests and zero unscanned uploads exposed for download.
- Communication guardrail: no more than one reminder per action per 24 hours and three automated reminders total unless staff explicitly send another.

Instrument only aggregate, privacy-respecting product events after consent. Do not record action titles, instructions, client names, filenames, comments, tokens, or file content in analytics or logs. Pilot outcome metrics come from tenant-scoped counts and an optional owner survey.

### Pricing and entitlement

The free experience is the account-free demo. Persistent use is a monthly subscription sold by the Sociobot billing service, with Dodo as merchant of record:

| Tier | Price | Included |
|---|---:|---|
| Starter | $49/month | 5 active client workspaces, 3 staff seats, 10 GB retained files, all action types, reminders, audit export |
| Studio | $99/month | 20 active client workspaces, 10 staff seats, 50 GB retained files, custom sending name, priority export |

Both plans include accessible experiences, data export, retention controls, and security behavior. These are never paywalled. “Active workspace” means a workspace with at least one open action during the billing period. Closed workspaces remain readable and exportable. There is no annual plan or per-client fee at launch.

The external API contract supplied to this work order documents a one-time license flow, while this brief requires recurring subscriptions. M2 must use the Sociobot recurring entitlement contract registered by the factory; it must not pretend a one-time license is a subscription. The application talks only to `https://api.sociobot.in/api/v1/...` in production and `https://pilot-api.sociobot.in/api/v1/...` in test mode. It never calls Dodo directly. The operator must register both recurring prices and the production return URL before billing can be claimed live.

### Deliberately out of scope

- Project plans, kanban boards, internal task assignment, time tracking, CRM, proposals, and general chat.
- Document editing, proof annotations, file collaboration, or cloud-drive replacement.
- Embedded payment collection or appointment scheduling. An action may point to the firm’s existing HTTPS payment or booking page and record that the client followed it; it never claims payment occurred without a verified integration.
- Legally regulated or qualified e-signatures. An approval records intent and an audit trail, but UI and terms must say it is not a regulated e-signature service.
- Passwords or a product-specific identity store. Staff use Sociobot Entra CIAM. Low-risk clients use scoped links.
- SMS, WhatsApp, marketing email, or open-ended campaigns.
- AI in the core workflow. Clear templates and deterministic rules solve the initial job more cheaply and predictably.

## 2. Evidence and wedge

### Demand signals

| Date | Source | Observed signal | What it supports | Limit |
|---|---|---|---|---|
| 2026-03-02 | [HN item 47217750](https://hn.algolia.com/api/v1/items/47217750) | A solo developer described HoneyBook as expensive and built a white-label portal after fragmented tools caused pain; a commenter asked for zero operational overhead. | Small firms will build or buy one client-facing place when broad suites and fragmented tools cost too much attention. | Anecdotal discussion; no willingness-to-pay sample. |
| 2026-06-05 | [HN item 48408917](https://hn.algolia.com/api/v1/items/48408917) | A five-year freelancer reported multi-client work becoming messy across tasks, documents, knowledge, and client platforms; clients liked one secure portal. | Both staff and client value consolidation. | The described portal is broader than this wedge. |
| 2025-01-06 | [Odoo issue 192439](https://github.com/odoo/odoo/issues/192439) | A 27-reaction request says hard-coded portal blocks make hiding, filtering, and renaming client data costly. | Client-facing scope and presentation must be configurable without custom modules. | Odoo users may have more complex needs than the target customer. |

Two independent freelancer/agency implementations plus a high-interest ERP complaint show recurrence. They do not yet prove the proposed price or that a focused action inbox beats a broad portal. The pilot experiments in the risk register must test those points.

### Incumbents and workaround

Customers currently stitch together email links, Drive or Notion folders, Slack, invoice pages, and project trackers. HoneyBook, Clientjoy, Odoo, and project-management products offer broad portals. Self-hosted portals reduce tool switching but add operational work.

### Wedge

The switch is from “open the client portal and find the right place” to “open one expiring link and complete the next action.” Client Action Room exposes a configurable, deadline-ordered action inbox and nothing resembling the firm’s internal project system. That yields:

- less client navigation and no client account for low-risk actions;
- less accidental disclosure because each link has an explicit scope;
- a retained, exportable answer instead of evidence spread across inboxes;
- a five-minute setup target rather than portal implementation work.

The defensible learning is not generic file storage. It is which request shape, deadline, reminder timing, and client surface get an external action completed.

## 3. Architecture

### Stack decision

- **Web:** Svelte 5, Vite, strict TypeScript, platform History API, and small headless primitives built in-repo. The queue has enough form, route, and async state to benefit from Svelte; React’s larger ecosystem is not needed. Initial JavaScript budget is 150 KB gzip on public routes and 200 KB gzip in the signed-in app.
- **API:** Rust 2021, axum, Tokio, serde, sqlx, tower/tower-governor, tracing, and PostgreSQL in production. Strong types and explicit concurrency suit tenant boundaries, idempotent submissions, uploads, and audit records.
- **Local/runtime fallback:** the container must start with only `PORT`. If `DATABASE_URL` is absent, use SQLite at `DATA_DIR/client-action-room.sqlite3` (default `/data`, falling back to a writable process directory for local development), generate any internal signing material with a CSPRNG, and log which settings were generated without values. Production sets PostgreSQL and object storage.
- **Deploy target:** one non-root multi-stage container for Azure Container Apps, fronted by factory ingress. The API serves the built web shell and `/api/*`; no cross-origin application API is needed. Static assets use hashed immutable caching.
- **Files:** private, region-bound Azure Blob-compatible object storage in production; quarantine and clean containers are separate. Local development uses a filesystem adapter below `DATA_DIR`. No bucket is public.
- **Malware scanning:** ClamAV-compatible scanner service. Uploads remain quarantined until a scan job records `clean`. Missing or failed scanning fails closed.
- **Email:** a factory-approved transactional provider behind an outbox adapter. Only action links and reminders; workspace owners opt in, clients can suppress reminders, and no marketing mail is sent.
- **AI:** none in M1–M4. M5 may test “draft an action from pasted email” only if interviews show demand. If built, it uses the Sociobot gateway, an explicit send preview, `gpt-5.6-*` discovery, a hard daily spend cap, recorded demo/test responses, and a manual form fallback. Raw Azure endpoints or keys are forbidden.

The decision and visual implications are also recorded in `.factory/design.md`.

### Repository contract

```text
/
├── .factory/                 product contract, claims, demo and handoffs
├── .github/workflows/        tests and reproducible builds
├── src/                      Svelte web application
│   ├── lib/components/       typed component contracts and implementations
│   ├── lib/domain/           pure state and validation logic
│   ├── lib/routes/           route definitions, titles and guards
│   └── styles/               design tokens and global foundations
├── tests/                    web unit/contract tests
├── e2e/                      Playwright claim and journey tests (from M1)
├── server/
│   ├── migrations/           reversible sqlx migrations
│   ├── src/routes/           edge validation and response mapping
│   ├── src/db/               tenant-scoped repositories
│   ├── src/jobs/             outbox, reminder, expiry and scan workers
│   └── tests/                API and isolation tests
├── public/                   metadata, icons and original image exports
└── dist/                     reproducible web build output
```

The current scaffold has a health-only API and a non-product placeholder page. M1 replaces the placeholder; it does not build around it.

### Runtime topology and request boundaries

```text
Browser
  ├─ public landing / demo shell ───────────────┐
  ├─ Entra PKCE for staff identity              │
  └─ same-origin /api requests                  ▼
Factory ingress → axum web/API container → PostgreSQL
                                  ├──────→ private object storage
                                  ├──────→ ClamAV scanner
                                  ├──────→ transactional email adapter
                                  ├──────→ Sociobot billing API
                                  └─ M5? → Sociobot AI gateway
```

Ingress terminates TLS. The app trusts the first `X-Forwarded-For` hop only when the direct peer is the configured ingress; otherwise it uses the peer address. Security headers apply to every HTML/API response. CSP starts with `default-src 'self'`, adds the Entra authority for auth only, the Sociobot billing/gateway origins only on routes that use them, and never permits unsafe inline scripts.

### Tenancy, identity, and roles

- A staff user is keyed by Entra `oid`, never email. Email and display name are mutable profile attributes.
- Every persistent business row contains `organization_id`, directly or through a foreign key whose repository method always receives an organization ID.
- Roles are `owner`, `admin`, and `member`. Owners manage billing and deletion; admins manage people, branding, links, and exports; members manage actions in assigned workspaces.
- External clients are not staff users. A client access grant has a workspace/action scope, allowed verbs, expiry, and revocation time.
- Authorization happens in the repository/service layer as well as the route guard. Tests create two organizations and attempt every cross-tenant read/write.

Sociobot Entra CIAM configuration defaults:

- Tenant ID: `35c6fe40-0ec0-46b6-98c6-213ad4de6650`
- Authority: `https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/`
- SPA client ID / token audience: `25c704f4-465a-47af-80ab-2c489466b697`
- Discovery: `https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/v2.0/.well-known/openid-configuration`
- Redirect: `https://client-action-room.sociobot.in/auth/callback`
- Scopes: `openid profile email`; MSAL cache: `sessionStorage`; flow: redirect + PKCE.

The API loads the discovery issuer and JWKS URI at startup, caches keys for one hour, and validates RS256, `aud`, `tid`, discovery `iss`, `exp`, and `nbf`. Invalid credentials return `401` with `WWW-Authenticate: Bearer`. Optional overrides use only `ENTRA_TENANT_ID`, `ENTRA_TENANT_SUBDOMAIN`, and `ENTRA_CLIENT_ID`.

### Scoped client link design

Generate 32 random bytes, encode base64url, and store only a versioned SHA-256 digest. The shared URL carries the secret in the fragment (`/client#access=<token>`), so browsers and mail scanners do not send it in HTTP requests. On load, the web app exchanges it once through `POST /api/client-links/exchange`, clears the fragment with `history.replaceState`, and receives a short-lived, `Secure`, `HttpOnly`, `SameSite=Strict` scoped cookie. The exchange checks expiry, revocation, allowed action IDs, and request rate before creating the session. Default link life is seven days; maximum is fourteen. Staff may expire it sooner or revoke it now.

Do not place secrets in query strings, structured logs, analytics, referrers, audit metadata, or support exports. Apply `Referrer-Policy: no-referrer` on client-link pages. State-changing client submissions require same-origin checks and an idempotency key. A link may view only its allowlisted workspace summary and actions.

### Data model

All IDs are UUIDv7 or another sortable, non-guessable server-generated ID. Timestamps are UTC; the workspace time zone controls display and reminder calculation.

| Entity | Important fields and rules | Owner / retention |
|---|---|---|
| `staff_user` | `entra_oid` unique, display name, email snapshot, last sign-in | Global identity; delete profile after last membership and retention window |
| `organization` | name, slug, default region, time zone, deletion state | Owner account |
| `membership` | organization, staff user, role, invited/accepted timestamps | Organization |
| `subscription` | organization, Sociobot customer/entitlement reference, tier, status, period end, verified at | Organization; provider tokens encrypted/server-only |
| `workspace` | organization, client label, optional client contact, brand settings, closed at | Organization; counts as active by open-action rule |
| `action` | workspace, kind, title, plain instructions, due time, status, version, created by | Organization; retained until owner policy/delete |
| `action_choice` | action, stable key, label, position | Parent action |
| `client_grant` | digest, scope, allowed verbs, expires/revoked at, last used | Organization; secret never stored |
| `submission` | action/version, grant, actor label, decision/choice/comment, idempotency key, server time | Immutable organization record |
| `file_object` | submission/action, object key, original filename, size, detected MIME, checksum, scan state, expires at | Quarantined then clean; delete by retention policy |
| `audit_event` | organization, workspace, action, actor kind/id, event name, timestamp, redacted metadata, hash-chain predecessor | Append-only; included in export |
| `reminder` | action, channel, scheduled/sent/suppressed timestamps, reason | Organization |
| `outbox_job` | kind, tenant, encrypted/redacted payload, attempts, available/locked time | Operational; delete after 30 days |
| `workspace` | namespace (`demo` or `real`), organization, labels, created/expires time | Demo rows hard-delete within 24 hours; real rows persist |
| `idempotency_record` | tenant/scope, key digest, request hash, response code/body, expiry | 24 hours unless the operation needs longer |

Audit events are application-append-only. Database permissions deny updates/deletes to the runtime path except the explicit whole-tenant erasure job. Hash chaining detects accidental mutation but is not advertised as tamper-proof.

### API and route conventions

- JSON uses a versioned `/api/v1` prefix. Errors use `{ "code", "message", "request_id", "field_errors?" }` and plain recovery text.
- Edge validation caps title at 120 characters, instructions at 2,000, client label at 80, comments at 1,000, choices at 12, and upload count at 10 per action. File size begins at 100 MB each and 250 MB per action; test demand before raising it.
- Every mutation accepts `Idempotency-Key`. Optimistic staff edits use an action version / `If-Match` and return `409` with the current record on conflict.
- Pagination is cursor-based, stable by `(due_at, id)` or `(created_at, id)`.
- External URLs must be HTTPS, normalized, and shown with their destination host. Private, loopback, link-local, non-HTTPS, and credential-bearing URLs are rejected.
- Server times are authoritative. UI renders the workspace time zone and always includes the date near relative phrases.

Planned API surface by M4:

```text
GET    /health
POST   /api/v1/demo/sessions
POST   /api/v1/demo/sessions/:id/reset
GET    /api/v1/demo/queue
POST   /api/v1/client-links/exchange
GET    /api/v1/client/actions
POST   /api/v1/client/actions/:id/submissions
POST   /api/v1/client/actions/:id/uploads
GET    /api/v1/me
POST   /api/v1/organizations
GET    /api/v1/workspaces
POST   /api/v1/workspaces
GET    /api/v1/workspaces/:id/actions
POST   /api/v1/workspaces/:id/actions
PATCH  /api/v1/actions/:id
POST   /api/v1/actions/:id/publish
POST   /api/v1/actions/:id/reminders
POST   /api/v1/actions/:id/client-links
DELETE /api/v1/client-links/:id
GET    /api/v1/workspaces/:id/audit
GET    /api/v1/organizations/:id/export
DELETE /api/v1/organizations/:id
POST   /api/v1/billing/checkout
GET    /api/v1/billing/entitlement
POST   /api/v1/billing/webhook
```

### Demo boundary

`/demo` and `?demo=1` call `POST /api/v1/demo/sessions`. The backend creates a random in-memory or database namespace with a 24-hour TTL, seeds fixed realistic data, and sets a separate demo cookie. No Entra token, billing record, production organization, email, blob, or AI request is accessible from that router. Demo writes are rate limited and discarded. “Reset demo” destroys the namespace and provisions the seed again. “Start for real” destroys it before beginning sign-in. Full sample and test rules are in `.factory/demo.md`.

### Billing boundary

The browser asks the API for checkout; the API maps the selected tier to a factory-registered Sociobot recurring product/price and returns the hosted checkout URL. The browser may then navigate only to the allowlisted Sociobot checkout origin. The return route re-checks entitlement server-side and never trusts query parameters as proof of payment. Webhooks are verified, idempotent, and treated as hints; a scheduled reconciliation remains authoritative. If billing is unreachable, existing customers keep their last valid entitlement for a 72-hour grace period and the free demo remains available.

Never store a Dodo key or call a Dodo endpoint. Secrets stay server-side. M2 tests use the pilot Sociobot service; production switches the base URL through configuration. The UI states price, recurring period, included limits, merchant of record, cancellation behavior, and links to `/privacy` and `/terms` before checkout.

### Background work

A database-backed outbox avoids a separate queue at launch. Workers claim jobs with `FOR UPDATE SKIP LOCKED` on PostgreSQL and an equivalent serialized lease on SQLite.

- `expire_actions`: every minute, mark link/action deadlines and append one event.
- `schedule_reminders`: every five minutes, enqueue eligible opt-in reminders in the workspace time zone.
- `send_transactional_email`: exponential backoff with jitter, maximum five attempts, redacted logs.
- `scan_upload`: pass quarantined object to ClamAV, record signature/version/result, then move clean files or delete rejected files.
- `purge_demo`: at least hourly; hard-delete expired demo sessions and objects.
- `apply_retention`: daily; delete expired files and requested tenant data with a deletion receipt.
- `reconcile_entitlement`: daily and after checkout return/webhook.

Job handlers are idempotent. A failed worker does not prevent the read/submit API from serving.

### Rate limits

`/health` may be exempt. Every other server route is limited by trusted client IP, plus tenant/grant where known. All exceeded limits return `429` with an integer `Retry-After` header.

| Boundary | Initial allowance |
|---|---:|
| Public page/API reads | 20 requests/second, burst 40 per IP |
| Demo session creation/reset | 3/minute per IP; 30 demo writes/minute per session |
| Entra callback/token-backed bootstrap | 10/minute per IP and `oid` |
| Client link exchange | 10/minute per IP; 5 failed tokens/minute |
| Client submission | 5/minute per grant and IP |
| Staff writes | 5/second, burst 10 per organization and user |
| Upload start/complete | 20/hour per grant; storage quota also enforced |
| Reminder send | 5/hour per organization; once/action/24 hours by product rule |
| Billing checkout | 5/hour per organization and user |
| Export/delete | 3/hour per organization |

M1 includes a load smoke that reaches a 429 and sees `Retry-After`. M2 repeats it behind a test `X-Forwarded-For` boundary.

### Observability and operations

- `/health` returns status and build SHA without probing dependencies. `/ready` (M2) checks migrations and required production adapters with a tight timeout.
- JSON logs contain UTC timestamp, level, build SHA, route template, status, duration, request ID, and pseudonymous tenant/user IDs. They exclude tokens, auth headers, query strings, action/client content, filenames, and email addresses.
- Metrics cover request count/latency, 429s, action transition counts, completion delay buckets, job age/failures, scan outcomes, object bytes, and email result. High-cardinality IDs are forbidden labels.
- Trace propagation uses W3C headers. Error reporting must scrub request bodies and secrets.
- SLO: 99.9% action read/submit availability. Page on a 10-minute fast burn or one-hour slow burn. Operations remain useful from logs/metrics without reading customer content.

### Backups, export, retention, and deletion

- PostgreSQL: encrypted point-in-time recovery with daily restore verification; target RPO 15 minutes and RTO 4 hours.
- Object storage: region-bound encryption, versioning for accidental deletion, and lifecycle rules matching workspace retention.
- Owner export: asynchronous ZIP containing CSV/JSON actions, submissions, audit events, workspace settings, and clean files with checksums. Export links expire in 24 hours and require owner re-authentication.
- Owner deletion: typed organization name confirmation, seven-day reversible queue, then tenant rows/objects/provider mappings are erased and a minimal non-content deletion receipt retained as law requires.
- Default clean-file retention is 90 days after completion; owner can choose 30, 90, 365 days, or retain until workspace deletion. Rejected/quarantined files are deleted immediately or within 24 hours.
- Region is selected before the first real workspace. The API rejects cross-region object configuration. Moving regions is an operator-assisted export/import until automated migration exists, and the UI says so.

### Privacy and security baseline

- Collect the minimum client label and actor label. Client email is optional unless email delivery is requested.
- Encrypt traffic and provider secrets; use platform encryption at rest. No secrets in the web bundle.
- Content Security Policy, HSTS, `X-Content-Type-Options: nosniff`, `Referrer-Policy`, `Permissions-Policy`, clickjacking protection, and cache controls are test assertions.
- Upload download responses use attachment disposition, detected safe MIME, and `X-Content-Type-Options`. Never serve quarantined files.
- Audit approval is not represented as a regulated e-signature. Terms and the approval screen repeat the limitation.
- No behavioral tracking. If a page counter is enabled, it is aggregate and contains no identifiers or action paths.
- Threat-model link theft, token logs, tenant ID substitution, IDOR, stored HTML, SSRF through external links/scanners, upload polyglots, replay, concurrent submission, mail abuse, and demo-to-production access before M2 exits review.

## 4. Design system

The visual source of truth is `.factory/design.md`; implementation tokens begin in `src/styles/tokens.css`, and the typed component contract is `src/lib/components/inventory.ts`.

### Direction

**Municipal archive window** treats the interface as a calm public records counter: warm docket paper, dark ink, oxidized bronze frames, narrow labels, and one service opening where the next action passes through. It is civic, accountable, and slightly uncanny—not nostalgic office decoration. The fixed frame represents the retained record; movable action slips represent work crossing between firm and client.

The interface is asymmetrical. A thick vertical “window frame” anchors the first screen while the live queue occupies the service opening. It must not become a centered gradient hero or a grid of generic feature cards.

### Token summary

- Palette: archive paper `#F1E8CF`, clean surface `#FFFAF0`, registry ink `#182522`, muted ink `#5A5B52`, bronze frame `#704A2D`, action teal `#165D56`, docket red `#963B30`, focus blue `#176E89`.
- Type: self-hosted, Latin-subset Newsreader semibold for display and Public Sans variable for body; combined WOFF2 budget 120 KB. System fallbacks work before fonts load.
- Scale: 4 px base, with 8/12/16/24/32/48/64/96 px working steps. Body is at least 16 px and 1.55 line height.
- Shape: 2 px stamps, 6 px tickets, 12 px service windows; strong straight rules and one clipped file-tab corner. Avoid pill-shaped controls except a true binary switch.
- Motion: an action slip moves no more than 12 px toward its recorded position over 220 ms. Completion applies one ink-stamp compression. No idle loops. Reduced motion uses instant state plus opacity.

Both light “public counter” and dark “closed stacks” treatments are specified and must maintain contrast. Status always includes a word and shape, never color alone.

### Component inventory

The 18-component inventory, states, accessibility behavior, and milestone ownership are in `.factory/component-inventory.md`. Required foundations include the page shell, demo banner, deadline rail, action slip, composer, client window, approval form, upload tray, choice field, audit ledger, status stamp, share-link panel, reminder control, notices, confirmation dialog, loading skeleton, and empty docket.

### Five key screens

1. **Landing / live preview:** the bronze frame cuts into the viewport from the left; a working three-action docket sits in the service opening. Copy and one sample-data action occupy the quiet paper area. Facts follow as docket lines, not cards.
2. **Staff action queue:** the deadline rail is the main reading order. Overdue, due soon, later, and complete groups use date tabs. The selected action opens as a detail sheet beside the rail on wide screens and as the next route on mobile.
3. **Action composer:** a narrow numbered form asks for action type, request, deadline, and link scope. A client-window preview updates beside it. The publish step names the expiry and exactly what the client can see.
4. **Client action window:** all staff chrome disappears. One request, deadline, firm identity, and one completion control sit within a high-contrast service opening. An expired link explains how to contact the firm without exposing other actions.
5. **Audit and settings:** a ledger-like chronological list sits beside restrained controls for retention, region, branding, staff, and billing. Export and delete are distinct, plainly named operations.

### States and responsiveness

- Empty: explain what will appear and offer one exact next step (“Create the first action”).
- Loading: reserve the final geometry; announce longer actions; do not use infinite decorative motion.
- Error: preserve input, state what failed, and provide one recovery action plus request ID when useful.
- Offline: public shell and already loaded content remain readable; writes say they require a connection. Do not claim full offline behavior.
- Mobile at 390 px: one rail or sheet at a time, bottom actions remain in document flow, due date stays beside status, and decorative frame depth reduces to one edge. Nothing essential depends on hover.
- Tablet: queue/detail may use a 40/60 split.
- Desktop: max content width 75 rem; the queue remains readable, not stretched. Copy measure stays at or below 68 characters.

### Accessibility rules

One `<h1>` and one `<main>` per route; skip link and landmarks on every page. Route navigation updates title, moves focus to the new heading, announces it politely, restores history and scroll, and preserves deep links. All targets are at least 44×44 px with 8 px separation. Forms have visible labels, described errors, and no placeholder-only labels. Dialogs trap and restore focus. Keyboard supports ordinary Tab/Enter/Space; custom queue movement, if added, follows documented arrow-key semantics without hiding tab stops. Contrast is at least 4.5:1 for text and 3:1 for UI/focus. Zoom to 200% and reduced motion are release tests.

### Site structure and titles

Header: wordmark to home; at most Demo, Pricing, Sign in, and Privacy; skip link first. Footer: one-line description, Privacy, Terms, “Built by Param Factory,” build ID, and generated-art disclosure only if generated art ships.

| Route | M | Title | Page `<h1>` / purpose |
|---|---:|---|---|
| `/` | 1 | `Client Action Room — get client actions done` | `Get client actions done on time` |
| `/demo` and `/?demo=1` | 1 | `Demo — Client Action Room` | `Your sample client action room` |
| `/client` after fragment exchange | 1 | `Client request — Client Action Room` | Dynamic action job, never the product name alone |
| `/privacy` | 1 | `Privacy — Client Action Room` | `How Client Action Room handles data` |
| `/terms` | 1 | `Terms — Client Action Room` | `Terms for Client Action Room` |
| `/404` | 1 | `Page not found — Client Action Room` | `We could not find this page` |
| `/auth/callback` | 2 | `Signing in — Client Action Room` | `Finishing sign-in` |
| `/onboarding` | 2 | `Set up your firm — Client Action Room` | `Set up your firm` |
| `/app` | 2 | `Action queue — Client Action Room` | `Client action queue` |
| `/app/workspaces/:id/actions/new` | 2 | `New action — Client Action Room` | `Create a client action` |
| `/app/workspaces/:id/audit` | 4 | `Audit record — Client Action Room` | `Action history` |
| `/app/settings` | 2–4 | `Settings — Client Action Room` | `Firm settings` |
| `/app/billing` | 2 | `Plans — Client Action Room` | `Choose a plan` |

M1 produces `robots.txt`, `sitemap.xml`, SVG favicon, 180 px touch icon, an original 1200×630 social image, and a correct `staticwebapp.config.json` or equivalent container fallback. No metadata references an asset that does not exist.

## 5. Milestones

Every milestone fits one focused builder session, ends in a deployable increment, and is followed by independent review → polish → PASS. Builders make small commits and update this status section plus a milestone handoff.

| Milestone | Status | Shippable outcome |
|---|---|---|
| M1 — Public site and approval demo | Repair 2 complete in source; live verification pending | A stranger can complete the sample, and a signed-in firm can start an empty durable approval workspace. |
| M2 — Accounts, persistence, and subscriptions | Partial identity foundation only; subscriptions planned | A firm can sign in and return to an isolated workspace. |
| M3 — Files, choices, and external links | Demo validation only; real workspace delivery planned | Clients can complete each action type through a scoped link. |
| M4 — Reminders, records, and operations | Demo scheduling only; delivery and operations planned | Staff can schedule a reminder and inspect its audit event. |
| M5 — Growth and integrations | Planned | Firms can import/share templates and connect action outcomes without turning the product into a project suite. |

### M1 — Public site and approval demo

**Status:** Repair 2 complete in source on 2026-09-05. The live deployment and independent verification still must pass.

**User outcome:** without an account, a visitor can use realistic sample data to experience the smallest complete action loop.

**Routes/screens added:** `/`, `/demo`, `/?demo=1`, `/client` after token-fragment exchange, `/privacy`, `/terms`, `/404`; consistent header/footer; demo banner on every demo/client sandbox state.

**Backend added:** isolated ephemeral demo session with 24-hour TTL; deterministic seed; approval action create/publish; scoped link exchange; approve/request-changes submission; deadline ordering; append-only audit events; reset/destroy; `/health`; endpoint-wide limits and security headers. The demo router has no code path to persistent organizations, email, billing, production blobs, or AI.

**Sample:** Northline Studio / Alder Street Bakery launch. Actions: approve the final menu proof due today, choose one of three launch-photo crops due tomorrow, upload the signed allergen sheet overdue by one day, and visit a hosted invoice link due in three days. M1 makes the approval action interactive; later action types are clearly marked as sample preview, not working claims. The sample client is Maya Chen and the sample staff owner is Theo Grant.

**M1 and repair claims:** `.factory/claims.json` is authoritative. Repair 2 adds real-workspace isolation, demo privacy, and staff-token boundary checks.

**Tests:**

- Vitest: action state transitions, deadline ordering including time-zone boundaries, plain validation, route title map, demo seed reset.
- Rust: demo tenant isolation, token digest/exchange/expiry/revocation, idempotent submission, audit append, rate limit 429 + `Retry-After`, security headers, health SHA.
- Playwright: exactly one test tagged for each claim from a fresh browser context and `/demo`; landing → demo → publish → client link → approval → staff audit; reset; expired link; back/forward/deep link; mobile 390 px; keyboard-only; no serious/critical axe findings; no console/page errors; outgoing demo requests are same-origin.
- Build/performance: `npm test`, `npm run check`, `npm run build`; initial public JS ≤150 KB gzip, app JS ≤200 KB gzip, CSS ≤50 KB, fonts ≤120 KB, social/hero mobile asset ≤300 KB, Lighthouse mobile ≥90 performance and ≥95 accessibility, LCP <2.5 s, CLS <0.1.

**M1 definition of done:**

- Every declared claim test passes from its documented clean sandbox; claim wording matches visible copy and README.
- Demo banner always says “Demo — sample data, nothing is saved,” with working Reset demo and Start for real. Start for real clears the sample before shared-tenant sign-in and never copies sample data.
- One-handed 390 px and keyboard journeys complete. Empty/loading/error/expired/offline states are present and plain.
- Landing follows the required information order, has one `<h1>`, route-specific metadata, original assets with provenance, `/privacy`, `/terms`, crawlable good links, sitemap/robots, CSP and icons.
- API is rate limited except health, uses no required secret, produces structured logs without client content, and the container runs as non-root on `PORT`.
- `npm test`, `npm run check`, and `npm run build` pass from a clean clone; `dist/` exists; Docker build/health smoke passes.
- `.factory/copy-audit.md`, `.factory/handoff-m1.md`, screenshots/traces, performance numbers, and plan status are committed. Independent review/polish reaches PASS.

### M2 — Accounts, persistence, and subscriptions

**User outcome:** a firm owner can sign in, create a durable organization/workspace/action, share it, and start or manage a recurring plan.

**Routes/screens added:** `/auth/callback`, `/onboarding`, `/app`, `/app/workspaces/:id/actions/new`, `/app/settings` (firm, people, region, retention), `/app/billing`; landing Pricing and Sign in become live. `/demo` remains account-free and isolated.

**Backend added:** Entra discovery/JWKS validation; organization/membership/workspace/action/client-grant/submission/audit PostgreSQL migrations; SQLite no-env fallback; tenant repositories; optimistic concurrency/idempotency; recurring Sociobot billing checkout, return, webhook and reconciliation adapter; entitlement limits; owner delete queue foundation; `/ready`.

**Claims added with one tagged test each:** staff can sign in with the shared Sociobot account; each firm sees only its own rooms; a published action remains after reload; Starter is $49/month for five active workspaces; Studio is $99/month for twenty active workspaces; cancellation leaves export available through the paid period. Do not publish a billing claim until a pilot checkout and entitlement round trip pass.

**Tests:** MSAL frontend adapter tests; JWT fixture tests for signature/audience/tenant/issuer/time and key rotation; two-tenant API matrix; reversible migration up/down test; persistence browser reload; checkout allowlist/return/tamper tests; idempotent signed webhook fixture; entitlement grace/limit tests; no-secret startup; demo regression; 100 rps read smoke and rate-limit verification behind trusted/untrusted forwarded headers.

**Definition of done:** a new user reaches published real action in under five minutes; no password path exists; all tenant queries are isolation-tested; billing uses pilot Sociobot service and recurring entitlements rather than a fake local flag; prices and legal copy match checkout; callback URI registration is confirmed or named under operator actions; clean build/claims/accessibility/security/performance gates pass; `.factory/handoff-m2.md` and plan status are committed; review/polish reaches PASS.

### M3 — Files, choices, and external links

**User outcome:** staff can ask for a file, a single choice, or a visit to a named external payment/booking page; clients finish each through the same focused window.

**Routes/screens added:** action composer variants and client completion states within existing `/app/.../actions/new` and `/client`; staff file state and download surface; client upload tray; choice confirmation; external destination confirmation.

**Backend added:** `action_choice`, `file_object`, and typed submission migrations; private multipart upload; checksum/MIME/size validation; quarantine → ClamAV scan → clean promotion; signed short-lived download; storage quotas/retention; HTTPS external-URL validation and destination-host audit event. An external-link action records “link opened,” never “paid” or “booked.”

**Claims added:** clients can upload listed file types up to the visible limit; files cannot be downloaded before a clean scan; rejected files state the reason and next step; clients can choose one listed option; external links show the destination host before leaving. Each gets one fixture-backed claim test; malware uses EICAR only in isolated tests.

**Tests:** upload edge cases, MIME disagreement, oversize/quota, EICAR rejection, scanner outage fails closed, signed URL expiry, clean-file authorization, SSRF URL corpus, choice idempotency/concurrency, external-link language, two-tenant objects, keyboard upload fallback, mobile/browser claim journeys, demo canned scan without object storage.

**Definition of done:** no quarantined/rejected object is served; client receives immediate progress and recoverable errors; file retention is visible before upload; regulated e-signature and payment limitations are in context; demo completes all action types without real storage or third-party navigation; gates/handoff/review PASS.

### M4 — Reminders, records, and operations

**User outcome:** staff can see what needs attention, send or schedule restrained reminders, export the record, and control retention/deletion while operators can run the service without reading customer content.

**Routes/screens added:** `/app/workspaces/:id/audit`, full `/app/settings` retention/notifications/export/delete, staff/role management, operational empty/error states. Client reminder suppression lives on `/client`.

**Backend added:** outbox/leases; deadline expiry; reminder scheduler and transactional email adapter; immutable audit export; owner ZIP export; seven-day delete queue; retention jobs; entitlement reconciliation; structured metrics/traces; readiness; backup and restore runbook; region-bound storage checks; admin is operational telemetry only, never customer-content browsing.

**Claims added:** reminders name the open action and can be suppressed; automation sends at most one reminder per action per 24 hours; CSV/JSON export includes every action and audit event; export links expire after 24 hours; organization deletion enters a seven-day recovery window. Each has one tagged test with a fake clock/provider and no real email.

**Tests:** DST/time-zone reminders, opt-out, retry/idempotency, outbox crash recovery, email redaction, export schema/checksum/access/expiry, delete/cancel/purge, retention across rows/objects, owner/admin/member matrix, restore drill, metrics cardinality and log-secret scan, demo and prior claim regressions, mobile/axe/performance.

**Definition of done:** operator runbook proves deploy, migration, rollback, restore, scanner outage, mail outage, billing outage, and abuse response; RPO/RTO evidence recorded; export/delete work without support; audit and logs contain no forbidden content; all gates/handoff/review PASS.

### M5 — Growth and integrations

**User outcome:** a firm can reuse its best request patterns and send verified action outcomes to tools it already uses, without exposing its internal work or adding a board.

**Routes/screens added:** template library within composer; `/app/settings/integrations`; signed webhook delivery log with redacted payload preview; installable PWA prompt only after repeat use. Landing reflects only integrations actually shipped.

**Backend added:** tenant templates; CSV action import with dry-run; outbound webhooks for `action.completed`, `action.overdue`, and `file.clean`; per-endpoint HMAC secret rotation, retry/dead-letter, egress allowlist/SSRF protection; PWA manifest and carefully scoped service worker for public shell/read-only cached states.

**Claims added:** a template creates a prefilled draft; CSV import reports invalid rows before writing; webhooks are signed and retried; the installed shell opens previously loaded read-only content without a network. Each claim has one deterministic sandbox test.

**Conditional AI experiment:** interview five activated firms about turning a client email into a draft request. Build only if at least three independently name it among their top two setup pains. Then add an explicit “Draft from pasted email” action that previews exactly what is sent, streams a draft via the Sociobot gateway, never publishes automatically, supports undo, uses canned demo/test responses, falls back to manual entry, and has spend/rate caps. Otherwise ship no AI and record the negative result.

**Tests:** template versioning, CSV atomicity/dry-run, webhook signature/rotation/replay/retry/SSRF, service-worker version/update/offline-read rules, install metadata, all earlier claims and limits. Any AI path adds recorded fixture claim tests plus one operator-only live gateway smoke when a factory key is present.

**Definition of done:** integration docs have copy-paste verification examples; webhook secrets are shown once and rotatable; service worker never caches scoped link secrets or auth/API mutations; growth additions do not create project-management features; gates/handoff/review PASS.

## 6. Risks and experiments

| Risk / unknown | Why it matters | Experiment that retires it | Decision threshold / milestone |
|---|---|---|---|
| Firms may want a broad portal, not a focused queue. | The wedge could feel too small for $49/month. | Show a clickable M1 queue to 8 owners; ask them to create last week’s real requests and compare with their current flow. | At least 5 can represent ≥80% of client chases without asking for a board; before M2 pricing launch. |
| Clients may distrust no-account links. | Completion falls if links look unsafe or reveal too much. | Moderated phone test with 10 client-side participants; vary firm identity, expiry, and scope explanation. | ≥8 identify the firm/request and complete without help; zero believe other client data is visible; M1 polish. |
| $49/month may not match willingness to pay. | Subscription economics depend on repeated follow-up pain. | Four-week concierge pilot with 6 firms; show real price before use, then offer paid continuation. | At least 3 accept $49 or provide a narrower, consistent objection; before M2 production billing. |
| “40% fewer emails” may be hard to measure. | The success measure can become vague or invasive. | Owner enters a two-week baseline count; product counts reminder sends and completed actions; optional end survey. | ≥4 pilots provide usable baseline; otherwise use median staff touches/action and document the change. |
| Link forwarding could expose client content. | Account-free use increases bearer-token risk. | Threat model plus red-team tests for fragment/token leakage, scope, expiry, revocation, referrer and logs. | No token in server access logs/history after exchange and no cross-action read; M1. |
| Approval may be mistaken for legal signature. | Legal and customer harm. | Test approval/terms copy with 5 owners and counsel review if regulated customers appear. | 5/5 describe it as an audit record, not regulated signing; M1–M2. |
| Malware scanning may delay or reject legitimate files. | Upload is a key action and a security boundary. | Measure 100 representative files against local ClamAV; test outage and retry language. | p95 scan <30 s for 100 MB and false rejection <1%; otherwise reduce limit/type set; M3. |
| Data residency could be claimed beyond actual deployment. | Enterprise buyers rely on it. | Provision one test organization per offered region; automated checks assert DB/blob/mail paths and restore in-region. | Offer only regions with passing evidence; never use a generic selector as proof; M4. |
| Reminder automation could become spam. | Harms clients and sending reputation. | Pilot default cadence with suppression; inspect completion and complaint rate. | <0.1% complaint, no action over policy cap; otherwise manual-only; M4. |
| Sociobot recurring API may differ from the supplied one-time contract. | Billing cannot be honestly shipped from an invented endpoint. | Factory registers Starter/Studio recurring products in pilot and provides checkout, return, webhook, verify fixtures. | Full recurring start/cancel/expiry round trip passes before M2 claim; operator dependency. |
| Email/object adapters may be absent when container starts. | Runtime contract requires no required env. | Clean-container boot with only `PORT`; health and demo work; real adapters report unavailable only on protected features. | Startup and M1 demo pass; M1/M2. |
| AI drafting may add cost and review burden without improving completion. | Decorative AI distracts from the core loop. | Five activated-customer interviews and a paper prototype; compare action creation time and edits. | Build only at 3/5 top-two pain and ≥30% median time reduction; M5 conditional. |
| The frame metaphor may reduce mobile clarity. | A distinct style cannot obstruct action completion. | 390 px five-second comprehension and keyboard/zoom tests against a plain baseline. | ≥85% identify next action; no horizontal scroll at 200% zoom; every milestone. |

## 7. Release governance

Each milestone builder must:

1. Update milestone status only after the claim suite and DoD pass.
2. Keep `?demo=1` working from a clean browser and keep prior claim tests in CI.
3. Add one test for every new public claim and remove copy whose outcome cannot be tested.
4. Write `.factory/handoff-m<N>.md` with commit, routes, migrations, tests, screenshots/traces, metrics, operator actions, known gaps, and exact next-step assumptions.
5. Run review → polish → PASS before the next milestone begins.
6. Never hide a security, billing, region, email, or AI stub behind finished-looking copy.

Current planner handoff is `.factory/handoff.md`. M1 begins from the buildable scaffold and replaces the placeholder page.
