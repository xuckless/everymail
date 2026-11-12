# Frontend roadmap

The Frontend is the React + Vite UI users see, shared by the Tauri desktop and mobile apps and the web app, plus the Astro landing site. It renders the views the Rust engine sends and turns input into intents; it holds no business logic and owns only presentation state.
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
- [ ] Bring the Linux baseline down before adding screens: the release build with only the shell screen measured 333 MB PSS on Fedora (WebKitGTK 2.54, Wayland), and 237 MB with `WEBKIT_DISABLE_DMABUF_RENDERER=1`
      Find out what the DMABUF renderer costs in motion quality before turning it off

## Motion foundations

- [ ] Add Motion for shared layout, exits and gestures, importing only the parts used; presses, fades and dots stay plain CSS
- [ ] Record Motion's bundle size and WebView heap cost against the RAM limit
- [ ] Motion tokens as CSS variables: ease-out entrances, ease-in-out moves, dropdowns at 150-250ms, exits 20% shorter, row stagger 30ms, press scale 0.97, popover start scale 0.95, no bounce
- [ ] Every token reduces to a fade or nothing under `prefers-reduced-motion`
- [ ] Press primitive in CSS: scale to 0.97, no tween while held
- [ ] Popover primitive: enters from and exits to its trigger at 0.95, phase as one enum (opening, open, closing)
- [ ] Segmented control and switch primitives: the knob moves ease-in-out and stretches 12% while moving
- [ ] Icon and label swap primitive in CSS: crossfade with blur, old and new states overlapping
- [ ] Success pulse primitive in CSS, played only on an engine-sent success state
- [ ] Sign-in screen from the engine's sign-in view with Google and Microsoft
- [ ] One-time notice that the credential is kept in a file, shown when the engine's credential storage notice says so

## First account connected

- [ ] Decide: redraw the design where decisions moved past it. Covers the connect screen's provider list, the model list (Grok, Local model), the agent builder's triggers, system agents in the agents list with deactivate instead of delete and appended instructions, Quarantine, the briefing view, and the calendar screen, which has no design yet. Options: redraw each from the engine's views before building it, build from the existing design and redraw after. Blocks the connect screen, the agents list, the AI compose model popover, the new agent screen, the Models section, Quarantine, briefing and calendar
- [ ] Connect screen from `ConnectView`: connected accounts, provider grid, waiting-for-sign-in step, syncing step with progress and stats
- [ ] Steps overlap old and new states; a newly connected account pulses once

## Inbox and thread reading

- [ ] App frame of sidebar, list and reading pane, responsive as in the design (reading pane hides below 1180px, sidebar below 820px)
- [ ] Sidebar from the inbox view: compose, all inboxes, accounts, folders, calendar, briefing, agents with status dots, new agent
- [ ] For you, Updates and Reading tabs with one indicator that moves between them by `layoutId`
- [ ] Windowed list: report the visible range on scroll and render the returned window; the pixel scroll offset stays in TypeScript
- [ ] Rows enter with a 30ms stagger; exits and reorders animate by stable row id
- [ ] Rubber band past the list bounds
- [ ] Reading pane: summary, body iframe at the engine's URI, agent draft block with approve and edit with AI
      Bodies never enter the JS heap
- [ ] Body iframe height: listen for the size message the engine's body script posts (check its source and origin) and set the iframe height; needs TODO-rust-core: body height script
- [ ] Thread screen from `ThreadView`: collapsible messages, thread summary with its pending state, back to inbox
- [ ] Invitation block in a thread with Accept, Maybe and Decline from the engine's answer state, pulsing on success
- [ ] Keyboard: ⌘N compose, arrow keys and Enter send selection and open intents, Escape goes back
- [ ] Accessibility: list as a listbox with an active descendant, ARIA roles on tabs and segmented controls, visible focus ring, labels on icon buttons

## Write actions

- [ ] Archive: the row travels on an arc to the Archive folder, whose count pulses on the engine's success
- [ ] Star, snooze and archive buttons on the reading pane and thread
- [ ] Snooze popover rendering the engine's options from its trigger
- [ ] Composer: text held in TypeScript while typing and sent to the engine on a debounce; the saved draft is the engine's
- [ ] AI compose panel: model popover, quick replies, tone segmented control, instruction field; draft paragraphs settle with a stagger
- [ ] Send label crossfades to Sent and pulses on the engine's sent state
- [ ] Undo send: after Send an Undo control shows until the engine's undoable instant; its countdown is TypeScript presentation only
- [ ] Send later popover from its trigger with a date and time picker; a scheduled send can be edited or cancelled from the engine's scheduled state
- [ ] Attach opens the platform file picker through the engine client
- [ ] Offline and queued-action indicator from the shell view

## Briefing, Quarantine and calendar

- [ ] Briefing view from `BriefingView`: the latest briefing with what agents did, what waits, condensed newsletters and the day's calendar
- [ ] Quarantine folder in the list like any folder, from the engine's inbox view
- [ ] Calendar screen from `CalendarView`: one view over all calendars with the engine-sent range, selection, and create, edit, delete and answer controls

## Search

- [ ] Search field: keystrokes local until committed, then sent to the engine; results render in the same windowed list

## Agents and approvals

- [ ] Agents screen: agent list with status, detail with stats, recent runs that expand to their steps, run now, pause switch
- [ ] System agents in the list with a system marker; detail shows the fixed built-in instructions read-only, a field for appended instructions, and Deactivate in place of Delete
- [ ] Off state for summaries, AI compose and user agents when the engine reports no key, with a way to the Models section
- [ ] Agent notes panel on the agent detail: notebook and per-thread notes, editable, input held in TypeScript until committed
- [ ] Paused-at-cap state on an agent and a notice with the engine-sent reason
- [ ] Approval queue: approve pulses done and the item exits; decline exits; items show their expiry and an expired item exits as declined; "All clear" empty state
- [ ] New agent screen: name, instructions, triggers with filters and the calendar lead time as the engine's `AgentBuilderView` sends them, schedule dropdown, account switches, Off/Ask/Auto per action, new-recipient switch, create
- [ ] Dry run panel: reading state, then result rows with a stagger
- [ ] Create agent pulses and crossfades to "Agent is live"

## Settings and rules

- [ ] Settings screen with a section nav whose indicator moves between Accounts, Agent permissions, Rules, Models and Appearance
- [ ] Accounts section controls: disconnect a mailbox, delete the account, and export what only EveryMail holds, each behind a confirm popover and rendering the engine's progress and result
- [ ] Models section: a key field per provider with its status; the key is held in TypeScript until committed and cleared after the engine takes it
- [ ] Usage against caps shown in settings, with a field for the optional daily token budget
- [ ] Block list rows beside the rules, from the engine's rules section
- [ ] Rule rows with switches; a new rule enters with the row stagger
- [ ] Light and dark token sets from tokens.json; the System theme follows `prefers-color-scheme`
- [ ] Theme change revealed with a circular mask from the click point, a crossfade where View Transitions are missing
- [ ] Density applied through CSS variables from the engine-sent preference

## Web parity

- [ ] Web build serves the same UI with the Worker engine client; nothing above the engine client branches on the target
- [ ] Register the Service Worker that answers body and attachment URLs by asking the engine Worker for the bytes (needs TODO-rust-core: Service Worker message protocol); the iframe URL has the same shape as on desktop
- [ ] Web Push: the Service Worker asks for notification permission and passes the push subscription to the engine client
- [ ] Decide: where the web app is served relative to the landing page on mail.everyai.dev. Blocks the web deploy and the landing deploy
- [ ] Deploy the web app where the decision says (needs TODO-backend: the reverse proxy routes mail.everyai.dev)

## Landing page

- [ ] Astro workspace for the landing site in this repo's Bun workspaces, built in CI and using the design tokens
- [ ] Landing page from the design: hero, agents, permissions, providers, apps and the waitlist signup
- [ ] Waitlist form posts to the public Backend call, encoded with the generated protobuf code; its state is one explicit enum (idle, sending, joined, failed); needs TODO-backend: public waitlist RPC
- [ ] Deploy the landing site at mail.everyai.dev where the decision says (needs TODO-backend: the reverse proxy routes mail.everyai.dev)

## Packaging and measurement gates

- [ ] Measure mobile limits before building on them: WebView memory on iOS and Android, and how much sync the operating system allows in the background
- [ ] Tauri 2 mobile project for iOS and Android hosting the same Frontend and crate
- [ ] Phone layout of the inbox from the design's phone screen
- [ ] Phone layout of the thread
- [ ] Phone layout of agents
- [ ] Phone layout of new agent
- [ ] Phone layout of connect
- [ ] Windows build in CI submitted to the Microsoft Store, which delivers updates
- [ ] macOS build in CI submitted to the Mac App Store, which delivers updates (needs TODO-rust-core: the Mac App Store sandbox check)
- [ ] Linux release artifacts built in CI for the first release
- [ ] Decide: how the Linux install script verifies what it downloads. Blocks the install script
- [ ] Linux install script fetched with curl from the first release, which installs the app and also updates it
- [ ] Measure whole desktop app memory (WebView processes plus engine) with a 50k-message replica after visiting inbox, thread, agents and settings, Linux (WebKitGTK) first, then macOS and Windows
      Gate: 300-400 MB on every desktop
- [ ] Check blur, masks, view transitions and layout animations for dropped frames on WebKitGTK and fall back per primitive where they fail
- [ ] Record JS bundle size in CI
