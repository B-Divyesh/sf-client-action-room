# Demo sandbox contract

Status: **implemented and expanded in repair 1**

- Entry: `https://client-action-room.sociobot.in/demo` and `https://client-action-room.sociobot.in/?demo=1`
- Local entry after M1: `http://localhost:5173/demo`
- Lifetime: anonymous backend session, hard expiry at 24 hours
- Namespace: a random `demo_session` ID in dedicated SQLite demo tables; never an organization ID
- Browser state: demo-only secure cookie and optional keys prefixed `demo:client-action-room:`; never `sb_license:*`, MSAL, or real app storage
- Banner: `Demo — sample data, nothing is saved`, with `Reset demo` and `Start for real`
- Reset: destroy the current namespace, provision a fresh deterministic seed, and keep the user on `/demo`
- Exit: `Start for real` destroys demo state before leaving the sandbox.

## Seed data

- Firm: **Northline Studio**
- Client workspace: **Alder Street Bakery launch**
- Staff owner: **Theo Grant**
- Client actor: **Maya Chen**
- Workspace time zone: **America/New_York**

Use a fixed test clock in claim runs. Relative due dates are derived from seed time, never left stale in fixtures.

1. **Approve the final menu proof** — approve or request a change, due today.
2. **Upload the signed allergen sheet** — PDF-only, 5 MB limit, safety scan, overdue by one day.
3. **Choose the launch photo crop** — three fixed choices, due tomorrow.
4. **Open the launch invoice** — disclosed public HTTPS destination, due in three days. The demo records a visit, never a payment result.
5. Every open action can schedule a sample email reminder. No message leaves the sandbox.
6. Audit seed: action created and deadline set. Publishing, reminders, and completion append server-time events.

## Isolation invariants

- The demo router cannot construct a production organization repository or query staff memberships.
- Demo never sends email, creates billing checkout, writes production blobs, or calls the AI gateway.
- Uploaded bytes stay in the isolated database, are limited to 5 MB PDFs, reject the EICAR fixture, and expire with the session.
- Browser contexts receive different namespaces; a guessed or copied session ID cannot cross cookies/grants.
- Token/link flows use demo-only grants with the same permission logic as production, while persistence stays isolated.
- Expired namespaces return a recoverable reset screen and reveal no prior content.
- Purge runs at least hourly and deletes demo rows/objects no later than 24 hours.
- Rate limits apply by IP and demo session. Reset cannot be used to bypass the IP allowance.

The ten tests in `.factory/claims.json` run from fresh browser contexts using only this entry and seed. Their fixed clock is `2026-08-28T14:00:00Z`; production derives the same relative deadlines from the live server clock.
