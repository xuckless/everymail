# Rust engine roadmap

The Rust engine is one crate, native in Tauri on desktop and mobile and WASM in a Web Worker on web. It owns all application state: the Backend connection, session and credentials, sync, the local SQLite database, search, the offline queue, navigation and selection, drafts, body and attachment serving, and the `ui.v1` views it shapes for the Frontend.
Other parts: [TODO-backend.md](TODO-backend.md) (Backend), [TODO-frontend.md](TODO-frontend.md) (Frontend).
Tests sit next to the logic they cover and only cover hard, deterministic logic.

## Walking skeleton

- [x] Connection state as an explicit enum with one pure transition function
- [x] Emits `ui.v1.ShellView` bytes and accepts `ui.v1.Intent` bytes
- [x] reqwest for HTTP on desktop and WASM
- [x] Minimal platform adapter for transport and the async runtime
- [x] prost codegen with generated code not committed, clippy in CI
- [x] Tests of the connection transition function for every state and event pair, including retry from each failure
- [x] One dispatch function from the `Intent` oneof to transitions; an unset oneof is a decode error
- [x] CI lints the crate for `wasm32-unknown-unknown` and builds the WASM package
- [ ] A Check that never gets a response leaves the state in Connecting forever: add a request timeout as an explicit event
- [ ] Enable a reqwest TLS backend on desktop before the Backend is reached over https (native builds have none today)
- [ ] Tauri app manifest naming each command the webview may call; every command not named is denied
- [ ] CI check that fails when a registered Tauri command is missing from the manifest

## Session and local database

- [ ] Session credential in the OS keychain on desktop and mobile (macOS, Windows, Linux Secret Service, iOS, Android) behind the platform adapter
- [ ] On Linux without a keychain service the credential goes into a file only the user can read; test that the file is created with owner-only permissions
- [ ] Credential storage notice as an explicit enum (none, file fallback not yet told, told) persisted by the engine and sent in the shell view, so the app says so once
- [ ] rusqlite (bundled) added with the first local table; the database is not encrypted and no secret is stored in it
- [ ] Hand-written SQLite migrations applied in order at engine start and recorded in one table; the drift check lives in TODO-backend: generated drift check in CI
- [ ] SQLite on disk on desktop and mobile holds metadata and thread structure for all mail and body text for the last 90 days
- [ ] Session state as an explicit enum (signed out, signing in, signed in, expired) with one pure transition function and tests
- [ ] One Backend client attaches the session to every call and turns an expired session into the signed-out transition
- [ ] `ui.v1` sign-in view and intents: choose Google or Microsoft, open the provider page in the system browser, finish against TODO-backend: `AuthService`
- [ ] Per-screen queries: each view is built from queries scoped to the current screen and selection, replacing the load of the whole replica on every input (about 52 queries and a full in-memory copy today); measure with a 50k-message replica
      Fine at sample-data size; required before a real mailbox, for speed and for the RAM limit
- [ ] The Backend port's requests and updates defined as `api.v1` protobuf messages and generated with prost, replacing the Rust types in `engine/src/backend.rs`
      Found by the audit: protobuf is the single source of truth for every type shared by the Backend and the engine, and today the port is Rust-only
- [ ] Time zones: wall times formatted in the user's zone and sent to the Backend as wall time plus an IANA zone; everything is UTC today
- [ ] Demo Backend removed from the shipped engine once the real Backend serves the port; until then it is the only implementation and lives only in `engine/src/demo/`
- [ ] Audit leftovers: the demo reads the whole replica through one broad interface; the first sync arrives as the seed's SQL text instead of typed frames; stable ids on briefing lines, news items, dry-run results, result groups, triggers and stats; the remaining unspecified-enum branches in body theming and provider mapping; frozen text in seeded stats
- [ ] Contract gaps the Frontend still works around: subject words for a rule arrive as one phrase; the custom interval's applicability is read from choice ids; a new rule has no default condition; an empty signature is not normalised to none; run outcomes carry no label

## First account connected and mail synced read-only

- [ ] `ui.v1.ConnectView`: connected accounts with status, the provider list (Gmail, Outlook, iCloud Mail, Yahoo Mail, Fastmail, Other IMAP), and the flow as a `oneof`: picking, waiting for provider sign-in, syncing with progress (messages, folders, search indexing or ready), failed
- [ ] Open the provider sign-in page in the system browser through the platform adapter, and finish connect when the Backend confirms it ("I've signed in" re-checks)
- [ ] IMAP server, user and app password passed once to the Backend, never stored by the engine
- [ ] Stream reader for TODO-backend: `SyncService.Stream` behind the platform adapter (native streaming on desktop, fetch streams on WASM), decoding length-prefixed frames
- [ ] Frame decoding tests: a frame split across reads, several frames in one read, a heartbeat, a truncated stream
- [ ] Stream connection as an explicit enum (connecting, streaming, waiting to reconnect with backoff) with one pure transition function; reconnect opens with the last cursor; tests of the backoff
- [ ] Sync loop: each Backend batch and its cursor written to SQLite in one transaction; a snapshot batch (newest mail first) is written the same way before live changes
- [ ] Older mail paged in with TODO-backend: `SyncService.ListOlder` when the list reaches the end of the replica
- [ ] Sync merge tests: a replayed batch, a delete before its insert, a message moved between mailboxes, a cursor reset forcing a full resync
- [ ] Per-account sync status as an explicit enum, surfaced in the shell view
- [ ] The inbox view is served as soon as the first batch lands

## Inbox and thread reading

- [ ] Navigation as one enum: inbox (all inboxes or one account, folder, tab), thread, calendar, briefing, agents, new agent, connect, settings section; persisted across relaunch
- [x] Navigation path: one pure function from the navigation enum to a web path (`/u/<n>/` then the view and, for a thread, its id) and its inverse from a path to an intent; an unknown path is an explicit not-found state; round-trip tests
- [ ] `ui.v1.InboxView`: sidebar (all inboxes with unread count, accounts with counts, Starred, Sent, Archive with count, Quarantine, agents with status and waiting count, the folders the user created), the For you, Updates and Reading tabs, and a window of rows
      Categories, Quarantine and folders come from Gatekeeper's sorting, synced from TODO-backend; nothing is derived from provider folders here
- [ ] Row view: stable id, sender, account glyph, subject, row note, time label, unread, starred
- [ ] Windowed list: the Frontend reports the visible range, the engine returns that window plus overscan; the scroll anchor is a row id held by the engine and kept across relaunch
- [ ] Window tests: anchor row removed, rows inserted above the anchor, range past the end
- [ ] Selected thread and the reading pane as engine state; which row is selected after an archive is decided here
- [ ] Time labels ("9:12", "Yesterday", "Mon 4:12 pm") formatted in the user's locale and time zone, with tests
- [ ] `ui.v1.ThreadView`: subject, account and message count, thread summary as a `oneof` (pending, ready, off without a key), messages each collapsed or expanded, per-message summary, an invitation block with answer state, a pending agent draft with its approval state
- [ ] Opening a thread asks the Backend for its summary through TODO-backend: `MailService` summary request; Scribe runs only then
- [ ] Allowlist HTML sanitizer applied when a body is served, on the parsed parts the Backend sends, with tests on hostile fixtures
- [ ] Body height script: the engine appends one small trusted script to each served body, allowed by a per-response nonce, that posts the body's height to the page
      Test that no other script survives: script tags, event handler attributes and `javascript:` URLs in hostile fixtures are gone, and only the nonced script remains
- [ ] Bodies served by message id through a Tauri custom URI scheme into a sandboxed iframe, with inline images resolved by the same scheme
      Raw content bytes only: bodies never enter the JS heap and never travel as protobuf or JSON
- [ ] Served bodies carry a CSP that allows only the nonced script, no top navigation; links open in the system browser
- [ ] A body older than the local window is fetched with TODO-backend: `MailService.GetBody` and served by the same scheme
- [ ] Attachments fetched with TODO-backend: `MailService.GetAttachment`, served through the same scheme by attachment id and saved through the platform adapter

## Write actions and the offline queue

- [ ] Offline queue in SQLite: each action stored with an idempotency key, sent in order per thread, retried with backoff, removed only on Backend acknowledgement
- [ ] Queued actions apply to the local replica at once and revert if the Backend rejects them
- [ ] Queue tests: order per thread, resume after relaunch, a duplicate acknowledgement, a rejection that reverts
- [ ] Intents for archive, star, unstar, mark read, mark unread, move, delete and snooze, sent per TODO-backend: `MailService` mutations
- [ ] Mark read when a thread opens, through the queue
- [ ] Snooze options in the view (Later today, Tomorrow, This weekend, Next week), sent as the preset plus the user's IANA zone to TODO-backend: `MailService.Snooze`
      The Backend resolves the time with its own clock; the engine shows the instant the Backend returns
- [ ] Offline state and queued-action count in the shell view
- [ ] `ui.v1.ComposerView`: from (account), to, subject, quoted thread; text arrives from the Frontend on a debounce and the engine persists the draft in SQLite
- [ ] Draft sync: each saved draft is sent to TODO-backend: `DraftService` and drafts from the user's other devices arrive on the sync stream; nothing is written to the provider's Drafts folder unless TODO-backend: Decide what to offer for Google accounts says so
- [ ] The account's signature inserted into new drafts
- [ ] Send queues through the offline queue; send state as a `oneof` (queued, sending, undoable until an instant the Backend stamps on receipt, scheduled for an instant, sent, failed)
- [ ] Intents for undo send, send later (wall time plus IANA zone, or an instant), and edit or cancel a scheduled send, sent to TODO-backend: `MailService.Send`
      Undo for a send still in the offline queue cancels the queue entry here; the 10 second window starts when the Backend receives it
- [ ] Attachments: on desktop the engine reads the picked file by path so the bytes never enter the JS heap, then uploads per TODO-backend: attachment upload
- [ ] Key state in the shell view as a `oneof` (no key, key set); with no key, summaries, AI compose and user agents are off while system agents run their code stage only
- [ ] AI compose view: model choice, quick replies, tone, instruction, and generation as a `oneof` (empty, writing, draft ready, failed) with the draft as paragraphs; calls TODO-backend: `ComposeService.Write`
- [ ] Approve and send, edit and decline on an agent draft go through the queue to TODO-backend: `ApprovalService`
      The engine relays the decision and makes none of its own

## Search

- [ ] SQLite FTS5 index of subject, sender, recipients and body text, updated in the same transaction as the mail row
- [ ] Local eviction of body text older than 90 days, in the same transaction as the removal from the search index
- [ ] Search query, filters and results as engine state, served through the same windowed list
- [ ] Search tokenizer and ranking tests on fixed fixtures
- [ ] Search by meaning: the engine calls TODO-backend: search by meaning and merges the results into the windowed list

## Agents and approvals

- [ ] `ui.v1.AgentsView`: agents with status as a `oneof` (running, done, waiting, paused, paused at a cap with the reason, deactivated, failed), a system marker, and next run; the selected agent's schedule, model, instructions, stats and recent runs with steps
- [ ] System agents in `AgentsView`: built-in instructions shown read-only with the user's appended instructions, per-action permissions and schedule editable, intents to deactivate and activate (delete only for user agents), sent to TODO-backend: `AgentService`
- [ ] Agent notes in `AgentsView`: the agent's notebook and its per-thread notes, with an intent to edit one, sent to TODO-backend: `AgentService` notes
- [ ] Approval queue view: items with agent, time, title, detail, reason and expiry, each item's state as a `oneof` (waiting, resolving, done, declined with a reason such as expired), and the waiting count for the sidebar
- [ ] Intents for approve, edit, decline, run now, pause and resume, sent to TODO-backend: `AgentService` and `ApprovalService`
- [ ] `ui.v1.AgentBuilderView`: name, instructions, all eight reactive triggers with sender, account, folder (the watch folder) and agent filters, a custom interval in the schedule options, the lead time for calendar event approaching, schedule options, accounts, Off/Ask/Auto per action, the new-recipient toggle, and the dry run as a `oneof` (idle, reading N messages across M accounts, results, failed)
- [ ] `ui.v1.BriefingView`: the briefings from News Bearer, newest first, from TODO-backend: `BriefingService`
- [ ] News in `BriefingView`: a news section per briefing with items grouped by topic, each with title, source and link, and an explicit empty state when the user has no topics
- [ ] News topics on News Bearer's detail in `AgentsView`: the topic list with intents to add and remove one, sent to TODO-backend: news topics
- [ ] The unfinished new agent persisted as an engine draft until created or cancelled
- [ ] Agent and approval changes reach an open app through the sync protocol
- [ ] `ui.v1` chat view on the agents screen for agents that can chat: the conversation's turns with each turn's state as a `oneof`, references that open a thread or an event, and the chat's availability as a `oneof` (ready, turn running, off without a key, agent deactivated)
- [ ] Engine-owned selection of the agent's activity or chat pane, with its web path
- [ ] Intents to send a chat message and to clear the conversation, sent to TODO-backend: `ChatService`; a message being sent is an explicit state on the turn (sending, sent, failed with a reason)
- [ ] Conversation synced into the local replica and updated through the sync protocol
      No permission or approval decision is duplicated from the Backend

## Settings and rules

- [ ] `ui.v1.SettingsView` with sections Accounts, Agent permissions, Rules, Models, Appearance; the selected section is engine state
- [ ] Accounts section: status ("Synced 1 min ago", "Sign-in expired"), signature, reconnect for an expired sign-in, add account
- [ ] Intent to disconnect a mailbox through TODO-backend: `AccountService`; the engine then deletes that mailbox's rows, queue entries and drafts from the local replica, with a test that none remain
- [ ] Intent to delete the account through TODO-backend: `AuthService`; the engine then deletes the whole local database and the stored credential and goes to signed out, with a test
- [ ] Intent to export what only EveryMail holds from TODO-backend: `UserDataService.Export`, saved through the platform adapter without the bytes entering the JS heap
- [ ] Models section: per provider (Anthropic, OpenAI, Google Gemini, OpenRouter) key status, and an intent that passes a new key once to TODO-backend: `ModelService`, never stored by the engine
- [ ] Usage against the caps, the optional daily token budget with an intent to set it, and the reason a cap stopped new runs, from TODO-backend: `SettingsService`
- [ ] Rules section with enabled toggles, new rule and the block list, sent to TODO-backend: `RulesService`
- [ ] Briefing times, pause all, default permissions and models per task, sent to TODO-backend: `SettingsService`
- [ ] Theme (System, Dark, Light) and list density (Compact, Comfortable, Roomy) as engine-owned preferences persisted locally and sent in the shell view

## Calendar

- [ ] Calendars and events in the local replica, filled from the sync stream
- [ ] `ui.v1.CalendarView`: events of the visible range from all calendars (Google Calendar, Outlook calendar, CalDAV) in one view; the range and the selected event are engine state
- [ ] Intents to create, edit and delete an event and to answer an invitation, also from a thread, sent to TODO-backend: `CalendarService`

## Notifications

- [ ] Desktop system notification raised from a notification event on the sync stream, through the platform adapter, while the app runs
- [ ] Web Push subscription and the APNs or FCM device token passed once to TODO-backend: `NotificationService`, never stored by the engine

## Web parity

- [ ] SQLite in memory on `wasm32` behind the platform adapter, one per tab, filled from the Backend on load; nothing persists in the browser and tabs do not coordinate
- [ ] A web tab opens the stream with no cursor, takes the snapshot of newest mail and pages older mail with `SyncService.ListOlder`, so it never pulls the whole mailbox
- [ ] No connection on web is an explicit state in the shell view, since there is no replica to read from; web has no offline queue, so an action taken without a connection fails at once with an engine-sent failed state
- [ ] Session on web is the HttpOnly cookie the Backend sets: requests go same-origin with credentials and the credential never enters WASM memory
- [ ] Service Worker message protocol: messages between the engine Worker and the Service Worker that carry bytes only, keyed by the same message and attachment ids as the desktop scheme

## Packaging and measurement

- [ ] Check what the Mac App Store sandbox constrains in the engine (file access, keychain, custom URI scheme) before the first store build, and record the findings
- [ ] SQLite `cache_size` and mmap limits set explicitly in one place
- [ ] Measure engine memory with a 50k-message replica and record it against the desktop budget (whole app measured in TODO-frontend)
- [ ] Record WASM bundle size in CI
- [ ] CI builds the crate for the iOS and Android targets
- [ ] Sync on iOS and Android within the background limits measured in TODO-frontend: mobile limits measurement
