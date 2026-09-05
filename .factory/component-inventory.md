# Component inventory

This is the build contract for the municipal archive window system. The typed mirror is `src/lib/components/inventory.ts`. Components remain `planned` until their milestone implements and tests them.

| Component | Purpose | Required states | Keyboard / screen-reader behavior | First milestone |
|---|---|---|---|---:|
| `PageShell` | Skip link, landmarks, route focus, header/footer | public, staff, client | Route title/focus/live announcement; back restores scroll/focus | M1 |
| `ArchiveHeader` | Wordmark, ≤4 nav actions, account context | public, signed-in, menu-open | Menu button announces expanded state; Escape closes/restores | M1 |
| `DemoBanner` | Persistent sample-data boundary | active, resetting, reset-error | Banner landmark; reset progress/result announced | M1 |
| `DeadlineRail` | Groups and orders actions by deadline | populated, empty, loading, error | Semantic headed lists; normal tab order; filters named | M1 |
| `ActionSlip` | One accountable action summary | open, due-soon, overdue, complete, disabled | Link/button semantics by action; status has word + shape | M1 |
| `ActionComposer` | Creates one approval request | draft, invalid, saving, saved, error | Labels/errors/summary; focus first invalid field | M1 |
| `ClientWindow` | Scoped client action surface | ready, submitting, complete, expired, revoked | One main action; scope and expiry read before form | M1 |
| `ApprovalForm` | Approve or request changes | ready, invalid, submitting, complete | Radio/fieldset or buttons with clear state; busy announced | M1 |
| `AuditLedger` | Append-only event history | populated, empty, loading, error | Chronological list; exact time in readable `<time>` | M1 |
| `StatusStamp` | Word/shape/color status | open, due-soon, overdue, complete, expired | Visible status word; decorative glyph hidden | M1 |
| `ShareLinkPanel` | Issue and copy one expiring link | none, active, copied, expired, error | Copy result live-announced | M1 |
| `InlineNotice` | Recovery/success/warning guidance | info, success, warning, danger | `status` for routine result; `alert` only for urgent failure | M1 |
| `ArchiveSkeleton` | Reserves final geometry | queue, detail, settings | `aria-busy`; decorative blocks hidden; no endless shimmer | M1 |
| `EmptyDocket` | Empty queue next step | new-workspace, filtered, complete | Heading and one exact next action | M1 |
| `ConfirmDialog` | Confirms sensitive action | closed, open, busy, error | Inert background, Escape when safe, focus trap/restore | M1 |
| `ChoiceField` | Captures one listed demo option | ready, invalid, submitting, complete | `fieldset`/radio semantics and clear validation | M1 repair |
| `UploadTray` | Selects one demo PDF and shows its scan outcome | empty, selected, scanning, accepted, rejected | Native file input remains available; result announced | M1 repair |
| `ReminderControl` | Records a reminder schedule without sending email | idle, scheduled, error | Schedule and result announced | M1 repair |

## Shared component rules

- Props accept domain states, not palette names.
- No component owns a customer-facing factual claim without a matching claim entry and tagged sandbox test.
- Busy controls keep their accessible name, expose state, and prevent duplicate writes through idempotency rather than visual disabling alone.
- Destructive controls use plain verbs and name the target in confirmation.
- Components meet 44×44 px targets, visible focus, dark/light/forced-colors contrast, reduced motion, 200% zoom, and 390 px layout checks.
- The typed mirror records which components are built and which remain planned; there is no public component-gallery route.
