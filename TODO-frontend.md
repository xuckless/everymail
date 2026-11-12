# Frontend roadmap

The Frontend is the React + Vite UI users see, shared by the Tauri desktop app and the web app. It renders the views the Rust engine sends and turns input into intents; it holds no business logic and owns only presentation state.
Other parts: [TODO-backend.md](TODO-backend.md) (Backend), [TODO-rust-core.md](TODO-rust-core.md) (Rust engine).

## Walking skeleton

- [x] Shell screen rendering the engine-sent connection state
- [x] CSS variables generated from the design system's tokens.json
- [x] Engine client with a Tauri implementation (raw-bytes invoke plus Channel) and a Web Worker implementation behind one interface
- [x] Biome in CI
- [ ] Presentation-state allowlist: one module declaring each TypeScript state item, its reason (updates within a frame, or input before commit) and its enum
- [ ] CI lint fails on `useState`, `useReducer` or any store outside the allowlist module
- [x] Each `oneof` in a view renders through an exhaustive switch the compiler checks; no state is derived from fields
- [x] Status labels overlap old and new states with no TypeScript state
- [ ] Confirm by sight that the shell renders in the Tauri window on each desktop OS (verified in a browser only so far)

## Motion foundations

- [ ] Decide: animation approach. Options: Motion (shared layout via `layoutId`, gestures; larger bundle and heap); CSS transitions with the Web Animations API and View Transitions (lighter, no shared layout). Check blur and view transitions on WebKitGTK first. Blocks every motion task below
- [ ] Motion tokens as CSS variables: ease-out entrances, ease-in-out moves, dropdowns at 150-250ms, exits 20% shorter, row stagger 30ms, press scale 0.97, popover start scale 0.95, no bounce
- [ ] Every token reduces to a fade or nothing under `prefers-reduced-motion`
- [ ] Press primitive: scale to 0.97, no tween while held
- [ ] Popover primitive: enters from and exits to its trigger at 0.95, phase as one enum (opening, open, closing)
- [ ] Segmented control and switch primitives: the knob moves ease-in-out and stretches 12% while moving
- [ ] Icon and label swap primitive: crossfade with blur, old and new states overlapping
- [ ] Success pulse primitive, played only on an engine-sent success state
- [ ] Sign-in screen from the engine's sign-in view

## First account connected

- [ ] Connect screen from `ConnectView`: connected accounts, provider grid, waiting-for-sign-in step, syncing step with progress and stats
- [ ] Steps overlap old and new states; a newly connected account pulses once

## Inbox and thread reading

- [ ] App frame of sidebar, list and reading pane, responsive as in the design (reading pane hides below 1180px, sidebar below 820px)
- [ ] Sidebar from the inbox view: compose, all inboxes, accounts, folders, agents with status dots, new agent
- [ ] For you, Updates and Reading tabs with one indicator that moves between them by `layoutId`
- [ ] Windowed list: report the visible range on scroll and render the returned window; the pixel scroll offset stays in TypeScript
- [ ] Rows enter with a 30ms stagger; exits and reorders animate by stable row id
- [ ] Rubber band past the list bounds
- [ ] Decide: how a sandboxed body iframe gets its height. Options: the engine's served document carries a trusted size reporter allowed by its CSP; a fixed-height iframe that scrolls; a size header from the engine. Blocks the reading pane (needs TODO-rust-core: body serving)
- [ ] Reading pane: summary, body iframe at the engine's URI, agent draft block with approve and edit with AI
      Bodies never enter the JS heap
- [ ] Thread screen from `ThreadView`: collapsible messages, thread summary, back to inbox
- [ ] Keyboard: ⌘N compose, arrow keys and Enter send selection and open intents, Escape goes back
- [ ] Accessibility: list as a listbox with an active descendant, ARIA roles on tabs and segmented controls, visible focus ring, labels on icon buttons

## Write actions

- [ ] Archive: the row travels on an arc to the Archive folder, whose count pulses on the engine's success
- [ ] Star, snooze and archive buttons on the reading pane and thread
- [ ] Snooze popover rendering the engine's options from its trigger
- [ ] Composer: text held in TypeScript while typing and sent to the engine on a debounce; the saved draft is the engine's
- [ ] AI compose panel: model popover, quick replies, tone segmented control, instruction field; draft paragraphs settle with a stagger
- [ ] Send label crossfades to Sent and pulses on the engine's sent state
- [ ] Attach opens the platform file picker through the engine client
- [ ] Offline and queued-action indicator from the shell view

## Search

- [ ] Search field: keystrokes local until committed, then sent to the engine; results render in the same windowed list

## Agents and approvals

- [ ] Agents screen: agent list with status, detail with stats, recent runs that expand to their steps, run now, pause switch
- [ ] Approval queue: approve pulses done and the item exits; decline exits; "All clear" empty state
- [ ] New agent screen: name, instructions, trigger segmented control, schedule dropdown, account switches, Off/Ask/Auto per action, new-recipient switch, create
- [ ] Dry run panel: reading state, then result rows with a stagger
- [ ] Create agent pulses and crossfades to "Agent is live"

## Settings and rules

- [ ] Settings screen with a section nav whose indicator moves between Accounts, Agent permissions, Rules, Models and Appearance
- [ ] Rule rows with switches; a new rule enters with the row stagger
- [ ] Light and dark token sets from tokens.json; the System theme follows `prefers-color-scheme`
- [ ] Theme change revealed with a circular mask from the click point, a crossfade where View Transitions are missing
- [ ] Density applied through CSS variables from the engine-sent preference

## Web parity

- [ ] Web build serves the same UI with the Worker engine client; nothing above the engine client branches on the target
- [ ] Body iframe on web per TODO-rust-core: Decide how bodies reach a sandboxed iframe on web
- [ ] Decide: landing page and waitlist (in the design: hero, agents, permissions, providers, apps, waitlist signup). Options: a static page in this repo, a separate site. Blocks the landing page and TODO-backend: waitlist signup RPC
- [ ] Landing page per the decision

## Packaging, updates and measurement gates

- [ ] Decide: mobile. The design has phone screens (inbox, thread, agents, new agent, connect). Options: Tauri 2 mobile with the same Frontend and engine, native shells over the engine, the web app only. Blocks phone work in all three parts
- [ ] Decide: desktop updates. Options: the Tauri updater with signed releases on own hosting, OS app stores. Blocks the first release
- [ ] Desktop bundles for macOS, Windows and Linux built in CI, signed and notarized
- [ ] Bring the Linux baseline down before adding screens: the release build with only the shell screen measured 333 MB PSS on Fedora (WebKitGTK 2.54, Wayland), and 237 MB with `WEBKIT_DISABLE_DMABUF_RENDERER=1`
      Find out what the DMABUF renderer costs in motion quality before turning it off
- [ ] Measure whole desktop app memory (WebView processes plus engine) with a 50k-message replica after visiting inbox, thread, agents and settings, Linux (WebKitGTK) first, then macOS and Windows
      Gate: 300-400 MB on every desktop
- [ ] Check blur, masks, view transitions and layout animations for dropped frames on WebKitGTK and fall back per primitive where they fail
- [ ] Record JS bundle size in CI
