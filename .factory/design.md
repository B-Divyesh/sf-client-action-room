# Client Action Room visual thesis

- Direction: **municipal archive window**
- Status: **implemented and retained through repair 1**
- Last updated: 2026-08-28

## Thesis

Client Action Room should feel like the one calm window in a public records hall: a clear place to hand over one thing, receive a dated record, and leave knowing what happens next.

The product’s subject is accountability between two parties. Its visual world therefore has two layers:

- a fixed, weighty frame for scope, deadlines, and retained records;
- light paper slips that cross the opening when an action moves from staff to client and back.

This is not an office-theme skin. There are no stock folders, fake paper clips, generic glass blur, marble halls, seal illustrations, or illegible rubber-stamp decoration. “Municipal” means public clarity and durable records. “Archive window” means a deliberately narrow service surface. The current request stays obvious within two seconds.

The visual system must be recognizable in a thumbnail: an off-centre bronze mullion divides warm ledger paper from a deep ink service opening, with deadline tabs projecting into the frame.

## Product and stack fit

Svelte 5 + Vite is the web choice because the product has a reactive deadline queue, multi-state forms, link exchange, and optimistic staff edits, without needing React’s ecosystem weight. Rust/axum + PostgreSQL owns tenant boundaries, expiring grants, idempotent submissions, file scanning, audit records, reminders, and billing. The design stays mostly CSS and semantic HTML; no runtime illustration or animation library is allowed without a measured need.

The public route targets 150 KB initial JavaScript gzip. The signed-in app targets 200 KB. The visual signature comes from composition, rules, type, and one optimized original illustration—not a heavy component kit.

## Visual grammar

### Composition

- The landing first screen uses an asymmetric 5/7 division at desktop. Copy sits on open ledger paper; the live sample queue sits inside the darker service opening.
- One 6 px “mullion” may connect hero, live preview, and the next section. It ends when it stops explaining the handoff. It is not repeated as decoration on every card.
- Queue items read like docket slips: a stable date/tab edge, a plain request title, client/workspace label, and status word. Independence comes from spacing first and a surface only where the slip can move or select.
- Horizontal ledger rules align related facts. Thick boxes are reserved for client scope, destructive confirmation, and the main service opening.
- Section rhythm alternates close procedural groups with generous 64–96 px pauses. It must not become a three-card SaaS grid.

### Shape language

- Archive stamp: 2 px radius.
- Docket ticket: 6 px radius with at most one clipped upper corner.
- Service window: 12 px radius inside a square bronze frame.
- Buttons are rectangular, at least 44 px high, and use verbs. Pills are limited to actual on/off switches.
- Status combines word, icon/shape, and color: open circle, due-soon half disc, overdue triangle, complete square stamp, expired crossed square.
- Shadows are shallow paper separation. The service window alone may use a deeper 18–50 px shadow.

### Texture

Texture is optional and CSS/SVG-made. A 2–3% monochrome paper grain may appear only on large empty paper areas. It must disappear in forced colors, not reduce contrast, and add under 5 KB. Never put texture behind small text, form inputs, or the client completion control.

## Palette

The palette comes from manila docket paper, carbon registry ink, oxidized bronze frames, faded blue routing slips, and red exception stamps. It avoids the blue-purple gradients and bright white cards common to generic SaaS sites.

### Light: public counter

| Token | Value | Use |
|---|---|---|
| `--color-canvas` | `#F1E8CF` | Warm ledger field |
| `--color-surface` | `#FFFAF0` | Forms and docket slips |
| `--color-surface-raised` | `#FFFDF7` | Dialog or selected sheet |
| `--color-text` | `#182522` | Registry ink |
| `--color-text-muted` | `#5A5B52` | Secondary copy on paper/surface |
| `--color-rule` | `#766F5E` | Rules and inactive outlines |
| `--color-frame` | `#704A2D` | Bronze mullion/frame |
| `--color-accent` | `#165D56` | Primary action and actionable links |
| `--color-on-accent` | `#FFFAF0` | Text on primary action |
| `--color-success` | `#2F6848` | Completed state with word/shape |
| `--color-warning` | `#80520D` | Due-soon state with word/shape |
| `--color-danger` | `#963B30` | Overdue/destructive with word/shape |
| `--color-focus` | `#176E89` | 3 px focus ring |

### Dark: closed stacks

| Token | Value | Use |
|---|---|---|
| `--color-canvas` | `#121B19` | Deep archive field |
| `--color-surface` | `#1D2926` | Queue/form surface |
| `--color-surface-raised` | `#26332F` | Selected sheet/dialog |
| `--color-text` | `#F4ECD8` | Warm paper text |
| `--color-text-muted` | `#C5BFAF` | Secondary text |
| `--color-rule` | `#8A8270` | Rules/outlines |
| `--color-frame` | `#C08B5A` | Lit bronze |
| `--color-accent` | `#82C9BD` | Primary action/link |
| `--color-on-accent` | `#10201D` | Text on primary action |
| `--color-success` | `#91CEA7` | Complete state |
| `--color-warning` | `#E2BD6B` | Due-soon state |
| `--color-danger` | `#F09889` | Overdue/destructive state |
| `--color-focus` | `#75CBE4` | Focus ring |

Before M1 handoff, run an automated contrast matrix against every actual token pairing. Text must reach 4.5:1 and focus/UI boundaries 3:1 in both modes. Muted text is never placed over imagery or frame colors without a surface.

Theme follows `prefers-color-scheme` on first visit and then an explicit user setting. The toggle is available from the header/settings, works before hydration without a flash, and persists only the theme preference.

## Typography

### Families

- **Newsreader Semibold**, display only. Its editorial cut suggests a public ledger without imitating a typewriter. Use for the single page headline and rare section titles, never dense UI.
- **Public Sans Variable**, body and controls. It was designed for clear public information and remains neutral beside the display face.
- System monospace, only for request IDs, file checksums, and developer-facing webhook examples.

Both chosen families use the SIL Open Font License. M1 must download from an official upstream release, retain the license files under `public/fonts/licenses/`, subset to required Latin glyphs, export WOFF2, and record source URLs, upstream commit/version, subset command, and file checksums below. Do not load Google Fonts or any CDN.

Combined font transfer budget: 120 KB. Use `font-display: swap`; preload at most the two above-the-fold files. Until the files ship, the tokens fall back to Georgia and the system UI stack.

### Scale and measure

| Token | Size | Typical use |
|---|---:|---|
| `--text-xs` | 12 px | Metadata only; never required body guidance |
| `--text-sm` | 14 px | Labels with adequate spacing |
| `--text-md` | 16 px | Body, controls, inputs |
| `--text-lg` | 20 px | Action title, subsection |
| `--text-xl` | 28 px | Page/section title |
| `--text-2xl` | 36–76 px fluid | Landing headline only |

Body leading is 1.55. Dense ledger rows may use 1.4. Headlines use 1.05–1.15. Long copy stays within 68 characters. Tables and dates use tabular figures. All interface copy uses sentence case.

## Spacing, grids, and elevation

Base unit: 4 px. Working scale: 4, 8, 12, 16, 24, 32, 48, 64, and 96 px. Components use 8/12/16/24. Page sections use 48/64/96. Do not invent in-between values without a documented optical correction.

- Mobile gutters: 16 px plus safe-area inset.
- Tablet gutters: 24–32 px.
- Desktop: 12-column fluid grid, maximum 1,200 px, 24 px gutters.
- Staff queue: one column under 720 px; 5/7 queue-detail split from 900 px.
- Touch targets: at least 44×44 px; adjacent targets have at least 8 px space.
- `--shadow-slip`: selected/movable paper only.
- `--shadow-window`: hero/client service opening only.
- Dialog backdrop uses solid ink at 72–78%; never rely on blur for legibility.

## Interaction grammar

The product has three verbs in its physical metaphor:

1. **Issue:** a draft slip enters the service opening when staff publish.
2. **Act:** the client control depresses like a firm counter button and immediately shows progress.
3. **Record:** the completed slip settles into the ledger and receives one status stamp.

Every interaction also works without motion. Hover never reveals required information. Pressed, busy, successful, failed, expired, and disabled states must be visually and programmatically distinct.

### Motion policy

- Standard transition: 220 ms using `cubic-bezier(0.2, 0.8, 0.2, 1)`.
- Small hover/focus feedback: 150 ms.
- Deliberate publish/record transition: at most 300 ms.
- Movement: transforms or opacity only; no more than 12 px for a slip and 4 px for a stamp compression.
- Landing frame may move up to 12 px once as the live preview enters. No continuous parallax or ambient loop.
- Loading uses a static reserved skeleton with a subtle one-time fade. No endless shimmer.
- Under `prefers-reduced-motion: reduce`, token durations become 0 ms; hierarchy, words, and focus remain sufficient.

## Signature scene and original assets

M1 creates one dominant hero/social scene: an impossible but orderly municipal service window in section. Oversized date tabs travel through a bronze frame from a busy paper ledger into a quiet client opening. No people, hands, logos, flags, official seals, or readable text appear in the art. The real UI sits in a solid high-contrast plate beside/within negative space; text never sits on the image.

Preferred implementation is a hand-made SVG composition because the subject is geometric and must stay small. If M1 instead generates a raster base, use the factory image generator and this prompt:

> Editorial architectural cutaway of an impossible municipal archive service window, oxidized bronze mullions, warm manila docket slips, deep carbon-green interior, oversized blank date tabs passing through one calm opening, precise civic wayfinding mood, asymmetrical negative space on left for interface, soft daylight, no people, no hands, no letters, no logos, no flags, no official seals, no gradients, 3:2.

Review raster output for unintended text, seals, civic insignia, duplicated tabs, impossible seams, and muddy negative space. Export AVIF/WebP at 640/960/1440 widths with explicit dimensions, a low-quality placeholder, mobile payload ≤300 KB, and meaningful alt text: “Client requests pass through one focused service window into a dated record.” The 1200×630 social card is composed from the same original scene with real HTML-equivalent metadata, never essential text baked into art.

### Provenance register

| Asset | Status | Source / author | License | Notes |
|---|---|---|---|---|
| Design tokens | Created in planner scaffold | Param Factory / Codex, 2026-08-28 | Repository MIT | `src/styles/tokens.css` |
| Hero/service-window scene | Shipped in M1 | Hand-authored SVG by Param Factory, 2026-08-28 | Repository MIT | `public/archive-window.svg`; no generated model; SHA-256 `328fd637ffd3e1e191b1cddda04b155a757d75fece4adfc00d8b0321eada7f2b`; reviewed at 390/1440 with no text, seals, brands, or seams |
| Social card | Shipped in M1 | Hand-authored SVG composition by Param Factory, with PNG export | Repository MIT | `social-card.svg` SHA-256 `a44c112983198464a16e5401763252e580f9ad7aa31f7f6b34bcb50b86186e4a`; `social-card.png` SHA-256 `46ad5865eb36a2eecd220b3eff6d1c858e3f7adc6a72c7d164d805b949d4ff6c` |
| Favicon/touch icon | Shipped in M1 | Hand-authored SVG by Param Factory; CairoSVG PNG export | Repository MIT | SVG SHA-256 `8767004830a90313deccc9e5465b04179c02c7f06776ef501c7af005499dc56b`; PNG SHA-256 `dc7b74ab5ceac7321525e1a18a6c0ec918086b73736b1643ecec6ac61261a017` |
| Status glyphs | Shipped in M1 | Hand-made CSS geometry by Param Factory | Repository MIT | One circle/half-disc/triangle/square family; no icon package |
| Newsreader subset | Shipped in M1 | Google Fonts repository commit `ade3d1533e06b2b1462ffcde8e08b129627ca360`; static 600/48 instance, Latin subset | SIL OFL 1.1 | Source `ofl/newsreader/Newsreader[opsz,wght].ttf`; `fonttools varLib.instancer`, then `pyftsubset`; WOFF2 SHA-256 `6891a63ea1552aaa1ce8439d9269a24acd4d49ddf812b3cedb60fc14363e1fbf` |
| Public Sans subset | Shipped in M1 | USWDS Public Sans repository commit `bae8aade44a1c1a2fdfeaabbfd6b6710b111a3a6`; variable Latin subset | SIL OFL 1.1 | Source `fonts/variable/PublicSans[wght].ttf`; `pyftsubset`; WOFF2 SHA-256 `9dfb099087354fd750c2dd9d7364e57284ee349986b2b2a9af60518523cca0d3` |

All M1 artwork is hand-authored. No generated imagery, stock media, external icon package, or runtime CDN ships.

## Component system

The canonical implementation inventory is `.factory/component-inventory.md` and the typed scaffold is `src/lib/components/inventory.ts`. General rules:

- Components receive semantic state, not raw colors.
- Every async component specifies idle, busy, success, error, and retry behavior where those states apply.
- Every destructive action is undoable or has a specific confirmation.
- Cards appear only for independent/movable records. Related settings group by heading/proximity.
- Icons have visible labels unless the meaning is universal and accessible name is still present.
- Dates show exact date/time near any relative label. Status never depends on position or color alone.

## Screen blueprints

### Landing and live preview

At 390 px, the headline and action appear first on uninterrupted paper. “Try it with sample data” is followed immediately by what happens. The three facts are ledger rows. The live queue then enters the single bronze-edged window. At desktop, copy occupies left negative space while the queue visually crosses the frame on the right. Subsequent sections follow: live product, three verb-led steps, boundary/privacy, exact plan price, footer.

### Staff deadline queue

The heading names the queue. A compact workspace selector and “Create an action” sit nearby but do not compete. Groups read overdue, due today, next, later, and complete. Each action slip exposes request, client, exact deadline, state word/shape, and primary next step. Filters are ordinary buttons/checkboxes. Selection opens a detail sheet whose origin is the selected slip.

### Action composer

A numbered vertical form uses these stable words: action, workspace, client link, deadline, reminder, audit record. The live client preview is secondary on mobile and side-by-side on desktop. Publish confirmation repeats what the client sees, what they can do, and when access ends. Draft recovery is explicit after network errors.

### Client action window

There is no app navigation. Firm wordmark/name, one request, exact deadline, instructions, and one form are visible. “What this link can see” opens a plain scope note. The completed state shows what was recorded and when. Expired/revoked/error states disclose no other workspace data and tell the client to ask the named firm contact for a new link.

### Audit, settings, and billing

Audit is a chronological ledger with filters and export, not an editable table. Settings use a calm one-column reading order on mobile. Region and retention explain consequences before selection. Billing compares Starter and Studio in aligned rows rather than decorative cards. Current tier, recurring price, period, limits, merchant, cancellation, and legal links are visible before checkout.

## Empty, loading, error, and offline states

- New queue: “No client actions yet. Create one request to get a client link.” Button: “Create an action.”
- Filtered queue: “No actions match these filters.” Button: “Clear filters.”
- Complete queue: “Every open action is complete.” Secondary line may state the next scheduled deadline if one exists.
- Loading: preserve rail/slip geometry, set `aria-busy`, and announce only if it exceeds 500 ms.
- Save error: “We could not save this action. Your text is still here. Check your connection and try again.”
- Expired link: “This client link has expired. Ask [firm] for a new link.”
- Offline read: “You are offline. This loaded copy is read-only until you reconnect.” There is no full offline claim.
- Scanner delay: “Your file is uploaded and being checked. Keep this page open or return with the same link.”
- Scanner rejection: name the safe category/recovery, not an opaque code; never expose engine internals.

## Responsive rules

- Design and test 390 px first, then 768, 1024, and 1440 px.
- Below 720 px, staff queue and action detail are separate history-aware views. The phone drops the ornamental second mullion, dense workspace metadata, and side-by-side preview. It keeps deadline, state, request, scope, and primary action.
- At 200% text zoom, layouts reflow without horizontal page scroll. Data tables become labeled definition rows rather than clipped grids.
- Fixed bars are avoided. If a completion control must remain available on long client instructions, use `position: sticky` only when it does not cover content, account for safe-area insets, and provide the same action in document flow.
- Images use explicit aspect ratios and source sizes. Below-the-fold media is lazy. The first meaningful queue/scene reserves space to keep CLS below 0.1.

## Accessibility contract

- Semantic header/nav/main/footer, one `<h1>`, ordered headings, lists for queues, real links and buttons.
- First element is a visible-on-focus skip link. Route changes update title, focus the heading with `preventScroll` where appropriate, announce it politely, and restore back/forward focus and scroll.
- Forms have visible labels, bound descriptions/errors, required text, and error summary. Focus goes to the summary after invalid publish, then to the first invalid field.
- Dialogs use a tested primitive, initial focus on the least destructive sensible control, Escape close when safe, inert background, and focus restoration.
- Three-pixel focus ring uses focus blue with an offset surface halo if needed. Forced-colors mode uses system colors and borders.
- Status shape + word + color. Charts are unnecessary; completion metrics use numbers and a text explanation.
- Axe serious/critical findings are zero. Keyboard, VoiceOver/NVDA smoke, 200% zoom, high contrast/forced colors, dark mode, and reduced motion are milestone handoff evidence.

## Metadata and product identity

Every route owns a title and plain description. Public page title: `Client Action Room — get client actions done` (44 characters). No title is just the product name or slug. Canonical URLs use `https://client-action-room.sociobot.in`. Open Graph/Twitter metadata references the real 1200×630 original scene. SVG favicon and 180 px touch icon use the bronze frame plus a single docket slit, with no letters too small to read.

The consistent site frame contains a home wordmark, at most four navigation items, and the footer required by the product contract. The 404 uses an empty service opening and the line “This record is not in the archive,” followed by an obvious home action.

## Implementation checks

Before each milestone handoff:

1. Compare actual styles to `src/styles/tokens.css`; remove untracked colors/spacing.
2. Capture 390 and 1440 px screenshots of all new routes and state variants.
3. Run the actual contrast matrix, axe, keyboard, reduced-motion, zoom, title/landmark/alt, console, and link crawls.
4. Measure compressed JS/CSS/fonts/images and Lighthouse budgets.
5. Update the provenance register for every new image, icon, and font.
6. Extract and audit public copy; each factual claim must map to `.factory/claims.json` and one tagged sandbox test.
