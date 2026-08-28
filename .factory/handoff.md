# Client Action Room handoff

M1 is built and deployed at <https://client-action-room.sociobot.in>. It ships the public site and a complete, isolated approval demo: publish a scoped link, answer as a client without an account, and read the immutable audit result.

## Verify

```sh
npm ci
npm run check
npm test
npm run test:e2e
npm run build

# Run the same browser claims against the deployed app
PLAYWRIGHT_BASE_URL=https://client-action-room.sociobot.in npm run test:e2e
```

All local gates passed from a clean clone. The production browser suite also passed all seven tests, including all six claims. Lighthouse mobile scored 99 performance, 100 accessibility, 100 best practices, and 100 SEO. The deployed container source is `612003b09b2513e79d32df7ed456355c430f5018`; `/health` reports it.

Detailed implementation, evidence, limits, and M2 requirements are in [`.factory/handoff-m1.md`](handoff-m1.md).

## Needs operator action before M2

- Register `https://client-action-room.sociobot.in/auth/callback` for the shared Sociobot Entra SPA.
- Register the $49/month Starter and $99/month Studio recurring prices in pilot and production, and supply the recurring Sociobot billing contract.

CIAM, persistent firm accounts, and recurring billing belong to M2 in the approved plan. They were not represented as M1 features. Independent review/polish remains the next gate; M2 must wait for PASS.
