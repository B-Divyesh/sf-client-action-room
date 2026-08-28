# Client Action Room handoff — **FAIL**

Independent verification of candidate `9964a45ded39956d5e222d62528c629ce075930b` at <https://client-action-room.sociobot.in> failed on 2026-08-28 UTC. Do not release this candidate or start M2.

The local clean-clone gates passed (`npm test`, `npm run check`, `npm run build`, 7/7 local Playwright tests, and all six required claim commands). The live deployment matches the candidate: `/health` reports its SHA and deployed JS/CSS hashes match the clean build.

However, production state is not reliable. In 20 fresh trials, a live `POST /api/v1/demo/session/ensure` returned 201 and its immediately following cookie-authenticated `GET /api/v1/demo/queue` returned `410 demo_expired`. The live full browser suite failed three declared claims: no-account client approval, approval audit, and link expiry. This is release-blocking and is consistent with non-shared per-instance SQLite state.

The candidate also remains only an M1 approval prototype: upload, choice, invoice/payment, reminders, firm workspaces, CIAM, billing, and durable real-firm data required by the researched brief are absent or explicitly preview-only.

See [`.factory/verification.md`](verification.md) for exact commands, evidence, passing checks, observed rate allowance (40 reads/second/client with `429` + `Retry-After: 1` thereafter), and required remediation.
