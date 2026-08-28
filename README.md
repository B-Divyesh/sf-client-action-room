# Client Action Room

Client Action Room is a planned action inbox for small agencies and service firms that chase client approvals, files, choices, and external payment or booking links.

The repository is at the planning-scaffold stage. It contains the venture contract, visual system, M1 claims, Svelte tooling, a health-only Rust API, and CI. It does **not** contain the client workflow or a usable demo yet. M1 will replace the placeholder page with the first complete demo flow.

Read:

- [`.factory/plan.md`](.factory/plan.md) for the PRD, evidence, architecture, milestones, tests, and risks.
- [`.factory/design.md`](.factory/design.md) for the municipal archive window visual system.
- [`.factory/claims.json`](.factory/claims.json) for the M1 claims that its sandbox tests must prove.
- [`.factory/demo.md`](.factory/demo.md) for the planned isolated sample data.

## Stack

- Svelte 5 + Vite + strict TypeScript for the web application.
- Rust 2021 + axum for the API; PostgreSQL is planned for shared production data.
- Sociobot Entra CIAM for staff identity and Sociobot billing for Dodo-backed recurring subscriptions. The product will never handle passwords or call Dodo directly.

## Develop

Requirements: Node 22+, npm, and the stable Rust toolchain.

```sh
npm ci
npm run dev          # web placeholder on http://localhost:5173
npm run dev:api      # health scaffold on http://localhost:8080/health
npm run check
npm test
npm run build        # web output in dist/; release API in server/target/
```

The Playwright version is pinned to 1.58.2 for the worker image. M1 adds browser specs and runs them through `npm run test:e2e`.

## Container

```sh
docker build --build-arg BUILD_SHA=local -t client-action-room .
docker run --rm -p 8080:8080 client-action-room
curl http://localhost:8080/health
```

The current container serves only `/health`; serving the built web shell is M1 scope. It starts with `PORT` alone and runs as a non-root user.

## Demo and deployment

The planned demo entry is `/demo` or `?demo=1`. It is not implemented in this planning work order. Production will ship at <https://client-action-room.sociobot.in>; the factory owns deployment, DNS, identity registration, and recurring-price registration.

## Privacy and terms

The scaffold stores no customer data and sends no analytics. M1 adds `/privacy` and `/terms` before any customer workflow. The product plan requires tenant isolation, scoped expiring client links, quarantined uploads, audit records, export/delete, and region-bound storage.

## License

[MIT](LICENSE) © 2026 Sociobot (Param Factory).
