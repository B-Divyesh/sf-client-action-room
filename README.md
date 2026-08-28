# Client Action Room

Give each client one short, deadline-based action list.

Client Action Room is for small agencies and service firms that chase approvals across email. M1 ships a complete, account-free approval demo. Staff issue a scoped link, a client answers, and the audit record shows the result.

Try the deployed sample at <https://client-action-room.sociobot.in/demo>.

## What M1 proves

- Try a ready client action room in one click.
- Demo changes are sample-only and resettable.
- A client can approve a request without creating an account.
- Open requests are ordered by deadline.
- An approval records the decision, actor label, and server time.
- An expired client link cannot read or submit the request.

The upload, choice, and external-link rows are labelled previews. Real firm accounts, CIAM sign-in, file handling, and monthly billing remain in their contracted later milestones.

## Demo boundary

The sample uses Northline Studio and the Alder Street Bakery launch. `/demo` creates a random server-side namespace that expires within 24 hours. Reset destroys that namespace and creates the same four sample actions again. Client link secrets stay in URL fragments and are exchanged for scoped HttpOnly cookies; SQLite stores only SHA-256 token digests.

The demo router cannot access future organization, billing, email, upload, or AI services. See [`.factory/demo.md`](.factory/demo.md) for its exact data and isolation rules.

## Stack

- Svelte 5, Vite, and strict TypeScript for the browser.
- Rust 2021, axum, and sqlx with SQLite for the M1 API.
- Reversible migrations under [`server/migrations`](server/migrations).
- A non-root multi-stage container that serves the API and built web application on `PORT`.

M2 will use the shared Sociobot Entra CIAM tenant for staff and the Sociobot recurring billing service for Dodo-backed subscriptions. This repository never handles passwords or calls Dodo directly.

## Run locally

Requirements: Node 22+, npm, and stable Rust.

```sh
npm ci
npm run build:web
DATA_DIR=.data-local DIST_DIR=dist PORT=8080 npm run dev:api
```

Open <http://localhost:8080/demo>. `PORT` is the only deployment variable required. `DATA_DIR`, `DATABASE_URL`, `DIST_DIR`, and `DEMO_FIXED_NOW` are optional overrides.

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

The image runs as a non-root user, creates its SQLite file under `/data`, applies migrations at startup, and serves `/health` with the build SHA.

## Deploy

The factory deploys the container to Azure Container Apps and owns DNS, CIAM redirect registration, secrets, and recurring-price registration. Do not deploy this repository as a static-only site because the isolated demo depends on its API.

## Privacy and legal

The M1 demo has no analytics or third-party runtime requests. Read `/privacy` and `/terms` in the application. Approval records capture intent but are not regulated electronic signatures.

## Product documents

- [Venture plan](.factory/plan.md)
- [Visual system](.factory/design.md)
- [Claims](.factory/claims.json)
- [M1 handoff](.factory/handoff-m1.md)

## License

[MIT](LICENSE) © 2026 Sociobot (Param Factory).
