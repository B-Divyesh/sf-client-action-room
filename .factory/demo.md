# Demo sandbox contract

Status: **implemented in M1**

- Entry: `https://client-action-room.sociobot.in/demo` and `https://client-action-room.sociobot.in/?demo=1`
- Local entry after M1: `http://localhost:5173/demo`
- Lifetime: anonymous backend session, hard expiry at 24 hours
- Namespace: a random `demo_session` ID in dedicated SQLite demo tables; never an organization ID
- Browser state: demo-only secure cookie and optional keys prefixed `demo:client-action-room:`; never `sb_license:*`, MSAL, or real app storage
- Banner: `Demo — sample data, nothing is saved`, with `Reset demo` and `Start for real`
- Reset: destroy the current namespace, provision a fresh deterministic seed, and keep the user on `/demo`
- Exit: `Start for real` destroys demo state before sign-in/onboarding. M1 explains that real accounts arrive in M2.

## Seed data

- Firm: **Northline Studio**
- Client workspace: **Alder Street Bakery launch**
- Staff owner: **Theo Grant**
- Client actor: **Maya Chen**
- Workspace time zone: **America/New_York**

Use a fixed test clock in claim runs. Relative due dates are derived from seed time, never left stale in fixtures.

1. **Approve the final menu proof** — approval, due today, open, interactive in M1.
2. **Upload the signed allergen sheet** — upload, overdue by one day, preview-only in M1 and interactive in M3.
3. **Choose the launch photo crop** — three realistic choices, due tomorrow, preview-only in M1 and interactive in M3.
4. **Open the launch invoice** — external link, due in three days, preview-only in M1 and interactive in M3; tests never navigate to a real third party.
5. Audit seed: action created, deadline set, and client link issued for the approval. Completing it appends the client decision.

Preview-only actions must say which milestone adds interaction. They may demonstrate queue ordering but cannot be described as working action types in public copy.

## Isolation invariants

- The demo router cannot construct a production organization repository or query staff memberships.
- Demo never sends email, creates billing checkout, writes production blobs, calls the AI gateway, or follows a real external payment/booking link.
- Browser contexts receive different namespaces; a guessed or copied session ID cannot cross cookies/grants.
- Token/link flows use demo-only grants with the same permission logic as production, while persistence stays isolated.
- Expired namespaces return a recoverable reset screen and reveal no prior content.
- Purge runs at least hourly and deletes demo rows/objects no later than 24 hours.
- Rate limits apply by IP and demo session. Reset cannot be used to bypass the IP allowance.

The six tests in `.factory/claims.json` run from fresh browser contexts using only this entry and seed. Their fixed clock is `2026-08-28T14:00:00Z`; production derives the same relative deadlines from the live server clock.
