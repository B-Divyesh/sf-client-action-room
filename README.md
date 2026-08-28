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
- An expired client link cannot read or submit the request.
- A client PDF is type-checked, safety-scanned, and scoped.
- A client can choose one listed option through a scoped link.
- A client sees the destination before opening an HTTPS payment or booking link.
- Staff can schedule one reminder and see its audit record.

The public sandbox runs all four action types without an account. Staff can use “Start for real” to sign in through the shared Sociobot Entra tenant and open an identity-isolated workspace.

## Demo boundary

The sample uses Northline Studio and the Alder Street Bakery launch. `/demo` creates a random server-side namespace that expires within 24 hours. Reset destroys that namespace and creates the same four sample actions again. Client link secrets stay in URL fragments and are exchanged for scoped HttpOnly cookies; SQLite stores only SHA-256 token digests.

The demo router cannot access organization, billing, email, or AI services. Sample PDFs stay inside the demo namespace and expire with it. See [`.factory/demo.md`](.factory/demo.md) for its exact data and isolation rules.

## Stack

- Svelte 5, Vite, and strict TypeScript for the browser.
- Rust 2021, axum, and sqlx with SQLite for the API.
- Reversible migrations under [`server/migrations`](server/migrations).
- A non-root multi-stage container that serves the API and built web application on `PORT`.

Staff authentication uses MSAL redirect with PKCE and session storage. The API validates RS256 tokens against discovered issuer and JWKS values, including audience and tenant. This repository never handles passwords, embeds provider secrets, or calls Dodo directly.

## Run locally

Requirements: Node 22+, npm, and stable Rust.

```sh
npm ci
npm run build:web
DATA_DIR=.data-local DIST_DIR=dist PORT=8080 npm run dev:api
```

Open <http://localhost:8080/demo>. `PORT` is the only deployment variable required. `DATA_DIR`, `DATABASE_URL`, `DIST_DIR`, `DEMO_FIXED_NOW`, and the documented Entra overrides are optional.

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

The factory deploys the container to Azure Container Apps and owns DNS, CIAM redirect registration, secrets, and recurring-price registration. Do not deploy this repository as a static-only site because the isolated demo depends on its API.

## Privacy and legal

The demo has no analytics. Only a client-chosen external action can leave the site. Read `/privacy` and `/terms` in the application. Approval records capture intent but are not regulated electronic signatures.

## Product documents

- [Venture plan](.factory/plan.md)
- [Visual system](.factory/design.md)
- [Claims](.factory/claims.json)
- [M1 handoff](.factory/handoff-m1.md)

## License

[MIT](LICENSE) © 2026 Sociobot (Param Factory).
