# Frontend roadmap

The Frontend is the React + Vite UI users see, shared by the Tauri desktop and mobile apps and the web app, plus the Astro landing site. It renders the views the Rust engine sends and turns input into intents; it holds no business logic and owns only presentation state.
Other parts: [TODO-backend.md](TODO-backend.md) (Backend), [TODO-rust-core.md](TODO-rust-core.md) (Rust engine).

## Walking skeleton

- [x] Shell screen rendering the engine-sent connection state
- [x] CSS variables generated from the design system's tokens.json
- [x] Engine client with a Tauri implementation (raw-bytes invoke plus Channel) and a Web Worker implementation behind one interface
- [x] Biome in CI
- [x] Presentation-state allowlist: one module declaring each TypeScript state item, its reason (updates within a frame, or input before commit) and its enum
- [x] CI lint fails on `useState`, `useReducer` or any store outside the allowlist module
- [x] Each `oneof` in a view renders through an exhaustive switch the compiler checks; no state is derived from fields
- [x] Status labels overlap old and new states with no TypeScript state
- [ ] Confirm by sight that the shell renders in the Tauri window on each desktop OS (verified in a browser only so far)
- [ ] Bring the Linux baseline down before adding screens: the release build with only the shell screen measured 333 MB PSS on Fedora (WebKitGTK 2.54, Wayland), and 237 MB with `WEBKIT_DISABLE_DMABUF_RENDERER=1`
      Find out what the DMABUF renderer costs in motion quality before turning it off

## Motion foundations

- [x] Add Motion for shared layout, exits and gestures, importing only the parts used; presses, fades and dots stay plain CSS
- [ ] Record Motion's bundle size and WebView heap cost against the RAM limit
      Bundle size recorded: about 144 kB (50 kB gzipped), the layout features loaded after first paint. WebView heap cost not measured
- [x] Motion tokens as CSS variables: ease-out entrances, ease-in-out moves, dropdowns at 150-250ms, exits 20% shorter, row stagger 30ms, press scale 0.97, popover start scale 0.95, no bounce
- [x] Every token reduces to a fade or nothing under `prefers-reduced-motion`
- [x] Press primitive in CSS: scale to 0.97, no tween while held
- [x] Popover primitive: enters from and exits to its trigger at 0.95, phase as one enum (opening, open, closing)
- [x] Segmented control and switch primitives: the knob moves ease-in-out and stretches 12% while moving
- [x] Icon and label swap primitive in CSS: crossfade with blur, old and new states overlapping
- [ ] Success pulse primitive in CSS, played only on an engine-sent success state
      The class exists in the design package; the app's fixture views never reach a success state, so it is unplayed there
- [x] Sign-in screen from the engine's sign-in view with Google and Microsoft
- [ ] One-time notice that the credential is kept in a file, shown when the engine's credential storage notice says so

## First account connected

- [x] Design redrawn to match the decisions: provider list without Proton, the model list, the agent builder's triggers, system agents in the agents list, Quarantine, the briefing view and the calendar screen
      Blocks the connect screen, the agents list, the AI compose model popover, the new agent screen, the Models section, Quarantine, briefing and calendar
- [ ] Connect screen from `ConnectView`: connected accounts, provider grid, waiting-for-sign-in step, syncing step with progress and stats
      Rendered from fixture views in every step; the connect controls beyond navigation have no intent yet
- [ ] Steps overlap old and new states; a newly connected account pulses once
      Steps overlap; no pulse yet, because the fixture flow adds no account

## Inbox and thread reading

- [x] App frame of sidebar, list and reading pane, responsive as in the design (reading pane hides below 1180px, sidebar below 820px)
- [x] One icon set of official company marks (mail, calendar and model providers, Google and Microsoft sign-in), used wherever an external company is identified; near-black marks follow the text colour
- [x] A thin line between sidebar groups, from the hairline token
      The owner asked for it: it is the stated reason this divider exists
- [x] Sidebar from the inbox view: compose, all inboxes, accounts, folders, calendar, briefing, agents with status dots, new agent
- [ ] For you, Updates and Reading tabs with one indicator that moves between them by `layoutId`
      Built with one CSS indicator from the design package, not `layoutId`
- [ ] Windowed list: report the visible range on scroll and render the returned window; the pixel scroll offset stays in TypeScript
- [ ] Rows enter with a 30ms stagger; exits and reorders animate by stable row id
      Enter stagger and exits by stable id are built; reorders need real data to be seen
- [ ] Rubber band past the list bounds
      Native bounce is kept where the platform has one (touch, macOS); nothing is added on Windows and Linux desktop, where it would fight native scrolling
- [ ] Reading pane: summary, body iframe at the engine's URI, agent draft block with approve and edit with AI
      Rendered; approve and edit with AI have no intent yet
      Bodies never enter the JS heap
- [x] Body iframe height: listen for the size message the engine's body script posts (check its source and origin) and set the iframe height; needs TODO-rust-core: body height script
- [ ] Thread screen from `ThreadView`: collapsible messages, thread summary with its pending state, back to inbox
      Rendered with collapsible messages and back; the summary has no pending state in the contract yet
- [ ] Invitation block in a thread with Accept, Maybe and Decline from the engine's answer state, pulsing on success
      Rendered from the engine's answer state; the answer buttons have no intent yet, so no pulse
- [ ] Keyboard: ⌘N compose, arrow keys and Enter send selection and open intents, Escape goes back
      Arrow keys and Enter are in on the inbox, Quarantine and agent lists. No ⌘N, because compose has no intent; no Escape, because no list screen has a back affordance
- [x] Accessibility: list as a listbox with an active descendant, ARIA roles on tabs and segmented controls, visible focus ring, labels on icon buttons
      Screen-reader output not checked

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

- [x] Briefing view from `BriefingView`: the latest briefing with what agents did, what waits, condensed newsletters and the day's calendar
- [ ] News section in the briefing view: items by topic with their source, each link opening in the system browser
- [ ] News topics editor on News Bearer's detail: add and remove a topic, input held in TypeScript until committed
- [ ] Quarantine folder in the list like any folder, from the engine's inbox view
      Built as its own screen from `QuarantineView` (list and detail with the body), not through the inbox view; Not spam and Delete now have no intent yet
- [ ] Calendar screen from `CalendarView`: one view over all calendars with the engine-sent range, selection, and create, edit, delete and answer controls
      Rendered: week view, phone agenda, event drawer and invitations with engine-owned selection; create, edit, delete and answer have no intent yet

## Search

- [ ] Search field: keystrokes local until committed, then sent to the engine; results render in the same windowed list

## Agents and approvals

- [ ] Agents screen: agent list with status, detail with stats, recent runs that expand to their steps, run now, pause switch
      Rendered with runs that expand to their steps; run now and the pause switch have no intent yet
- [ ] System agents in the list with a system marker; detail shows the fixed built-in instructions read-only, a field for appended instructions, and Deactivate in place of Delete
      Rendered; the appended-instructions field and Deactivate have no intent yet
- [x] Off state for summaries, AI compose and user agents when the engine reports no key, with a way to the Models section
      Both off states lead to the Models section; no fixture shows them
- [ ] Agent notes panel on the agent detail: notebook and per-thread notes, editable, input held in TypeScript until committed
- [ ] Paused-at-cap state on an agent and a notice with the engine-sent reason
- [ ] Approval queue: approve pulses done and the item exits; decline exits; items show their expiry and an expired item exits as declined; "All clear" empty state
      Rendered with waiting, done and all-clear states; approve and decline have no intent yet, and there is no expiry
- [ ] New agent screen: name, instructions, triggers with filters and the calendar lead time as the engine's `AgentBuilderView` sends them, schedule dropdown, account switches, Off/Ask/Auto per action, new-recipient switch, create
      Rendered on desktop and as the phone's steps; every field and control beyond navigation has no intent yet
- [ ] Dry run panel: reading state, then result rows with a stagger
      Idle, reading and result states render; the fixture shows only idle
- [ ] Create agent pulses and crossfades to "Agent is live"
- [ ] Chat pane on an agent that can chat (user agents and Assistant): turns from the engine's chat view, each state from its `oneof`, references opening their thread or event, new turns entering with the row stagger
      Rendered from fixture conversations in every turn state, with references opening their thread or event; the off and deactivated states are unseen
- [ ] Chat composer: text held in TypeScript until sent; disabled states come from the engine's availability `oneof`
      Rendered with the engine's availability states; Send has no intent yet
- [ ] Clear conversation behind a confirm popover

## Settings and rules

- [x] Settings screen with a section nav whose indicator moves between Accounts, Agent permissions, Rules, Models and Appearance
- [ ] Accounts section controls: disconnect a mailbox, delete the account, and export what only EveryMail holds, each behind a confirm popover and rendering the engine's progress and result
      Rows render; disconnect, delete and export have no intent or confirm popover yet
- [ ] Models section: a key field per provider with its status; the key is held in TypeScript until committed and cleared after the engine takes it
      Rendered with each provider's key state; the key goes nowhere yet
- [ ] Usage against caps shown in settings, with a field for the optional daily token budget
      The budget states render; the field goes nowhere yet
- [ ] Block list rows beside the rules, from the engine's rules section
      Rendered as its own section; Remove has no intent yet
- [ ] Rule rows with switches; a new rule enters with the row stagger
      Rendered; the switches and the new-rule form have no intent yet
- [x] Light and dark token sets from tokens.json; the System theme follows `prefers-color-scheme`
- [x] Theme change revealed with a circular mask from the click point, a crossfade where View Transitions are missing
      Checked in Chromium with View Transitions removed; not in a browser that really lacks them
- [x] Density applied through CSS variables from the engine-sent preference

## Web parity

- [x] Web build serves the same UI with the Worker engine client; nothing above the engine client branches on the target
- [ ] Register the Service Worker that answers body and attachment URLs by asking the engine Worker for the bytes (needs TODO-rust-core: Service Worker message protocol); the iframe URL has the same shape as on desktop
      Done for bodies; attachments not yet. On web the iframe is isolated by a response header, not the sandbox attribute (see docs/ENGINE.md)
- [ ] Web Push: the Service Worker asks for notification permission and passes the push subscription to the engine client
- [ ] One origin, mail.everyai.dev: the landing site at the root, the web app under `/u/<n>/` paths, the Backend's calls under `/api.v1.`
- [x] On web the address bar shows the path the engine sends, and a loaded or changed path is sent to the engine as an intent; no router in TypeScript (needs TODO-rust-core: navigation path)
- [ ] Deploy the web app where the decision says (needs TODO-backend: the reverse proxy routes mail.everyai.dev)

## Landing page

- [x] Astro workspace for the landing site in this repo's Bun workspaces, built in CI and using the design tokens
- [x] Landing page from the design: hero, agents, permissions, providers, apps and the waitlist signup
- [ ] Waitlist form posts to the public Backend call, encoded with the generated protobuf code; its state is one explicit enum (idle, sending, joined, failed); needs TODO-backend: public waitlist RPC
      Built and checked against a mock; states are idle, sending, joined, already joined, invalid and failed. Ends in failed until the Backend call exists
- [ ] Deploy the landing site at mail.everyai.dev where the decision says (needs TODO-backend: the reverse proxy routes mail.everyai.dev)

## Packaging and measurement gates

- [ ] Measure mobile limits before building on them: WebView memory on iOS and Android, and how much sync the operating system allows in the background
- [ ] Tauri 2 mobile project for iOS and Android hosting the same Frontend and crate
- [ ] Phone builds go to TestFlight and Play testing in CI; the public stores wait until the product is past alpha
- [x] Phone layout of the inbox from the design's phone screen
- [x] Phone layout of the thread
- [x] Phone layout of agents
- [x] Phone layout of new agent
- [x] Phone layout of connect
- [ ] Windows build in CI submitted to the Microsoft Store, which delivers updates
- [ ] macOS build in CI submitted to the Mac App Store, which delivers updates (needs TODO-rust-core: the Mac App Store sandbox check)
- [ ] Linux release artifacts built in CI for the first release
- [ ] Each Linux release publishes a SHA256SUMS file signed with a GPG release key; the install script checks the signature, then the hash, before installing
- [ ] Linux install script fetched with curl from the first release, which installs the app and also updates it
- [ ] Measure whole desktop app memory (WebView processes plus engine) with a 50k-message replica after visiting inbox, thread, agents and settings, Linux (WebKitGTK) first, then macOS and Windows
      Gate: 300-400 MB on every desktop
- [ ] Check blur, masks, view transitions and layout animations for dropped frames on WebKitGTK and fall back per primitive where they fail
- [x] Record JS bundle size in CI
