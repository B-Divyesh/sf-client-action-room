# Client Action Room — repair 1 handoff

Date: 2026-08-28 UTC

## Result

The two release-blocking findings from verifier commit `3a75e3c406f6632012bf6f5ad7abe1c540f6bff0` were reproduced and repaired.

1. The live `201 → 410 demo_expired` failure came from up to three Container App replicas, each with private SQLite state. The deployment now uses one writer, local SQLite, and an Azure Files snapshot at `/data`. Every successful API mutation publishes a snapshot; startup restores it. Old revisions are deactivated. Regression tests cover a request crossing two app instances on one database and snapshot restore.
2. Upload, choice, external-link, and reminder rows are no longer previews. Each has a scoped server route, validation, completion state, and server-time audit event. The upload accepts one PDF up to 5 MB, checks its real signature, rejects the EICAR fixture, stores a SHA-256 checksum, and expires with its isolated room. External actions disclose and validate the HTTPS host, and never claim payment. Staff sign-in uses the shared Sociobot Entra tenant; the API discovers issuer/JWKS and validates RS256, audience, tenant, expiry, and stable `oid` identity.

The existing approval, reset, deadline ordering, expiry, privacy, accessibility, and rate-limit behavior remains covered.

## Exact verification

From a clean dependency install:

```sh
npm ci
npm run check
npm test
npm run build
npm run test:e2e
```

Results:

- `npm ci`: 89 packages, 0 vulnerabilities.
- `npm run check`: Svelte/TypeScript 0 errors and 0 warnings; rustfmt clean; clippy clean with `-D warnings`.
- `npm test`: 5/5 web tests and 10/10 Rust unit/integration tests.
- `npm run build`: `dist/` produced; public JS 28.55 KB gzip, lazy auth JS 67.07 KB gzip, CSS 4.86 KB gzip; Rust release build passed.
- `npm run test:e2e`: 12/12 Chromium tests passed across desktop and 390×844 mobile, including all ten claim tags, keyboard focus, History API routing, request privacy, and axe serious/critical checks.
- Factory `verify-url.sh`: title, `lang`, one `h1`, `main`, image alt text, and console checks passed; desktop and 390 px screenshots were captured under `.factory/evidence/repair-final/`.
- Lighthouse mobile against the final live build: Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 1.4 s, CLS 0.
- Response policy: gzip is enabled; hashed assets return one-year immutable caching; API/client responses remain `no-store`; CSP, HSTS, `nosniff`, `no-referrer`, COOP, and Permissions Policy remain present.
- Identity discovery returned the tenant-GUID issuer and documented JWKS URI. Missing and forged bearer tokens return `401` plus `WWW-Authenticate: Bearer`.
- Final live build: `/health` reports `fea7944a8c52f1e33c3f980992dceb2364146445`; one active healthy revision, one replica, and the `/data` Azure Files mount.
- Live claim run: `PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e` passed 12/12.
- Exact verifier persistence probe after single-revision routing: 20/20 fresh `POST /api/v1/demo/session/ensure` → cookie-authenticated `GET /api/v1/demo/queue` cycles returned `201 → 200`.
- Durable restore probe: a fresh session returned four actions after an explicit restart of the live Container App revision.

`docker` is not installed in this worker. The factory ACR build is the container build evidence; it succeeded from the repository Dockerfile and started as the non-root runtime user.

## Deploy

Run:

```sh
.factory/deploy.sh
```

The wrapper uses the factory container deployer, creates or reuses the `client-action-room` Azure Files share, mounts it at `/data`, sets `DATA_DIR` to local storage and `PERSIST_DIR` to the durable mount, constrains scale to one replica, and deactivates older revisions.

## Operator dependencies

- Confirm that `https://client-action-room.sociobot.in/auth/callback` is registered on SPA client `25c704f4-465a-47af-80ab-2c489466b697`. The repository cannot inspect tenant app registrations.
- Register the recurring Starter ($49/month) and Studio ($99/month) products in Sociobot billing before exposing checkout. The documented production checkout currently returns `404 enabled factory product`, so no broken or misleading purchase control is shown.
- Production reminder delivery and a full ClamAV adapter remain provider operations. The shipped sandbox records reminder scheduling without sending email and proves only its explicit PDF/EICAR safety checks.

## Rollback

Deactivate the repair revision and reactivate the preceding healthy revision. The durable snapshot is isolated in the `client-action-room` Azure Files share. Database migrations have matching down files; do not downgrade a snapshot without first exporting it.
