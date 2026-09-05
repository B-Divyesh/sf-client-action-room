# Client Action Room

Give each client one short, deadline-based action list.

Client Action Room is for small agencies and service firms that chase client actions across email. Staff issue a scoped link, a client acts, and the audit record shows the result.

Try the deployed sample at <https://client-action-room.sociobot.in/demo>.

## What the product proves

- Try a ready client action room in one click.
- Demo changes are sample-only and resettable.
- A client can approve a request without creating an account.
- Open requests are ordered by deadline.
- An approval records the decision, actor label, and server time.
- Client links last seven days, then cannot read or submit the request.
- A client PDF is type-checked, malware-scanned, and scoped.
- A client can choose one listed option through a scoped link.
- A client sees the destination before opening an HTTPS payment or booking link.
- Staff can schedule one reminder and see its audit record.
- A firm starts with an empty, isolated workspace that persists.
- Demo traffic stays on this site, data lasts at most 24 hours, and leaving deletes the room.
- Staff workspace requests reject missing and invalid access tokens.

The public sandbox runs all four action types without an account. “Start for real” opens a signed-in, empty firm workspace for durable approval requests. File, choice, external-link, billing, and email delivery remain later firm-workspace milestones.

## Demo boundary

The sample uses Northline Studio and the Alder Street Bakery launch. `/demo` creates a random server-side workspace that expires within 24 hours. Reset destroys it and creates the same four sample actions again. Sample activity never moves into a signed-in firm workspace.

Client link secrets travel in URL fragments. The browser removes the fragment after exchange. See [`.factory/demo.md`](.factory/demo.md) for the exact sample and isolation rules.

## Stack

- Svelte 5, Vite, and strict TypeScript for the browser.
- Rust 2021, axum, and sqlx with SQLite for the API.
- Reversible migrations under [`server/migrations`](server/migrations).
- A non-root multi-stage container that serves the API and built web application on `PORT`.
- ClamAV with build-time signatures. The server records an upload only after a clean scan result.

Staff authentication uses the shared Sociobot Microsoft Entra tenant. Staff access rejects missing or invalid tokens.

## Run locally

Requirements: Node 22+, npm, stable Rust, and ClamAV for a real upload scan.

```sh
npm ci
npm run build:web
DATA_DIR=.data-local DIST_DIR=dist PORT=8080 npm run dev:api
```

Open <http://localhost:8080/demo>. `PORT` is the only deployment variable required. The container includes ClamAV. Local browser tests use a recorded scanner fixture.

For frontend-only work, run `npm run dev`; Vite does not proxy the Rust API.

## Test and build

```sh
npm run check
npm test
npm run test:e2e
npm run build
```

The Playwright suite starts the built application and pins the browser package to 1.58.2. Each public claim has one test tagged `@claim:<id>` in [`e2e/claims.spec.ts`](e2e/claims.spec.ts).

Run those same tests against an existing deployment with:

```sh
PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e
```

## Container

```sh
docker build --build-arg BUILD_SHA=local -t client-action-room .
docker run --rm -p 8080:8080 client-action-room
curl http://localhost:8080/health
```

The image runs as a non-root user, creates its SQLite file under `/data`, applies migrations at startup, and serves `/health` with the build SHA. The factory deployment mounts Azure Files and uses one writer replica.

## Deploy

The factory deploys one replica to Azure Container Apps with SQLite on `/data`. It owns DNS, CIAM redirect registration, and recurring-price registration. Do not deploy this as a static-only site.

## Privacy and legal

Demo traffic stays on this site. Only a client-chosen external action can open another site. Read `/privacy` and `/terms`. Approval records are not regulated electronic signatures.

## Product documents

- [Venture plan](.factory/plan.md)
- [Visual system](.factory/design.md)
- [Claims](.factory/claims.json)
- [M1 handoff](.factory/handoff-m1.md)

## License

[MIT](LICENSE) © 2026 Sociobot (Param Factory).
