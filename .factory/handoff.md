# Client Action Room — independent verification handoff

Date: 2026-08-28 UTC

## Result: FAIL

**Candidate:** `052905494eff58fe50bab41ec254b956a68ae353`
**Live URL:** <https://client-action-room.sociobot.in>

Do not release this candidate. Fresh live `/health` identifies the tested commit. The complete verification is in `.factory/verification-2.md`.

## Evidence summary

- Clean install, unit/API tests, Svelte/rustfmt/clippy checks, exact production build, local Playwright 12/12, and live Playwright 12/12 passed.
- All ten declared demo claim tests passed locally and live. The cold live first screen is plain-language and includes one-click **Try it with sample data**.
- Desktop and 390 px mobile are usable; axe reported no serious/critical findings; visible keyboard focus, reduced-motion behavior, same-origin request logs, candidate asset parity, cache policy, and security headers passed.
- The initial public JS is 28,421 bytes gzip, CSS 4,883 bytes gzip, and shipped fonts total 65,444 bytes.

## Blocking defects

1. **P0: rate limiting is not live.** 100 burst reads yielded 100 × 401 and no 429; five demo-session POSTs yielded 5 × 201 and no 429/`Retry-After`, despite documented 40 reads/sec and 3 session writes/min.
2. **P0: no actual firm product.** “Start for real” signs in then loads a long-lived record in `demo_*` tables seeded with Northline Studio/Alder Street Bakery. The UI explicitly states real accounts and monthly plans are unavailable. There are no real organization/workspace/action/audit-export/billing routes.
3. **P0: no malware scanner.** The upload path accepts any `%PDF-` file unless it contains `EICAR`, then marks it `clean`; this is not malware scanning or a fail-closed quarantine flow.
4. **P1: unlisted claims.** Several privacy/security/lifecycle claims in landing/privacy/README have no required observable claim test.

## How to verify

```sh
npm ci
npm test
npm run check
npm run build
npm run test:e2e
PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e
```

Docker image verification could not run because `docker` is unavailable in this verifier environment.

## Required next steps

Implement and verify rate limiting at the deployed ingress/runtime, a real firm data model/workflow, fail-closed malware scanning, and claims coverage before re-verification. No deployment action is recommended for this candidate.
