# Rust engine roadmap

The Rust engine is one crate, native in Tauri on desktop and WASM in a Web Worker on web. It owns all application state: the Backend connection, session and credentials, sync, the local SQLite database, search, the offline queue, navigation and selection, drafts, body and attachment serving, and the `ui.v1` views it shapes for the Frontend.
Other parts: [TODO-backend.md](TODO-backend.md) (Backend), [TODO-frontend.md](TODO-frontend.md) (Frontend).

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
- [ ] Decide: Tauri capabilities. Options: none, as today (app commands are allowed for the local origin, every plugin command is denied); an app manifest granting each command by name so new commands are denied by default

## Decisions that block everything

- [ ] Decide: sync protocol between Postgres and the local SQLite replica, including streaming. Options: cursor-based pull over unary POST with long-poll; server-streaming responses in the Connect streaming framing; WebSocket. Needs TODO-backend: Decide what the Backend stores per message. Blocks sync, live updates, sync progress and approvals reaching an open app
- [ ] Decide: replica depth. Options: all mail; a time window plus on-demand fetch; headers for all mail and bodies for a window. Blocks SQLite size, search completeness and initial sync time
- [ ] Decide: where the session credential lives. Options on desktop: OS keychain, encrypted file; on web: HttpOnly cookie, IndexedDB. Needs TODO-backend: Decide auth. Blocks sign-in
- [ ] rusqlite (bundled) added with the first local table, schema per TODO-backend: Decide schema ownership
- [ ] SQLite migrations applied in order at engine start and recorded in one table
- [ ] Session state as an explicit enum (signed out, signing in, signed in, expired) with one pure transition function and tests
- [ ] One Backend client attaches the session to every call and turns an expired session into the signed-out transition
- [ ] `ui.v1` sign-in view and intents

## First account connected and mail synced read-only

- [ ] `ui.v1.ConnectView`: connected accounts with status, the provider list (Gmail, Outlook, iCloud Mail, Yahoo Mail, Fastmail, Other IMAP, Proton per TODO-backend: Decide Proton Mail), and the flow as a `oneof`: picking, waiting for provider sign-in, syncing with progress (messages, folders, search indexing or ready), failed
- [ ] Open the provider sign-in page in the system browser through the platform adapter, and finish connect when the Backend confirms it ("I've signed in" re-checks)
- [ ] IMAP server, user and app password passed once to the Backend, never stored by the engine
- [ ] Sync loop per the sync protocol decision: each Backend batch and its cursor written to SQLite in one transaction
- [ ] Sync merge tests: a replayed batch, a delete before its insert, a message moved between mailboxes, a cursor reset forcing a full resync
- [ ] Per-account sync status as an explicit enum, surfaced in the shell view
- [ ] The inbox view is served as soon as the first batch lands

## Inbox and thread reading

- [ ] Navigation as one enum: inbox (all inboxes or one account, folder, tab), thread, agents, new agent, connect, settings section; persisted across relaunch
- [ ] `ui.v1.InboxView`: sidebar (all inboxes with unread count, accounts with counts, Starred, Sent, Archive with count, Receipts, agents with status and waiting count), the For you, Updates and Reading tabs, and a window of rows
- [ ] Row view: stable id, sender, account glyph, subject, row note, time label, unread, starred
- [ ] Windowed list: the Frontend reports the visible range, the engine returns that window plus overscan; the scroll anchor is a row id held by the engine and kept across relaunch
- [ ] Window tests: anchor row removed, rows inserted above the anchor, range past the end
- [ ] Selected thread and the reading pane as engine state; which row is selected after an archive is decided here
- [ ] Time labels ("9:12", "Yesterday", "Mon 4:12 pm") formatted in the user's locale and time zone, with tests
- [ ] `ui.v1.ThreadView`: subject, account and message count, thread summary, messages each collapsed or expanded, per-message summary, a pending agent draft with its approval state
- [ ] Bodies served by message id through a Tauri custom URI scheme into a sandboxed iframe, with inline images resolved by the same scheme
      Raw content bytes only: bodies never enter the JS heap and never travel as protobuf or JSON
- [ ] If TODO-backend: Decide where MIME parsing and HTML sanitizing happen puts sanitizing here, an allowlist sanitizer with tests on hostile fixtures
- [ ] Served bodies carry a CSP with no scripts and no top navigation; links open in the system browser
- [ ] Attachments served through the same scheme by attachment id and saved through the platform adapter

## Write actions and the offline queue

- [ ] Offline queue in SQLite: each action stored with an idempotency key, sent in order per thread, retried with backoff, removed only on Backend acknowledgement
- [ ] Queued actions apply to the local replica at once and revert if the Backend rejects them
- [ ] Queue tests: order per thread, resume after relaunch, a duplicate acknowledgement, a rejection that reverts
- [ ] Intents for archive, star, unstar, mark read, mark unread, move, delete and snooze, sent per TODO-backend: `MailService` mutations
- [ ] Mark read when a thread opens, through the queue
- [ ] Snooze options in the view (Later today, Tomorrow, This weekend, Next week) with each preset resolved here to an absolute time in the user's time zone, with tests
- [ ] Offline state and queued-action count in the shell view
- [ ] Decide: whether user drafts sync to the provider's Drafts folder through the Backend. Options: local only, synced. Blocks drafts on other devices
- [ ] `ui.v1.ComposerView`: from (account), to, subject, quoted thread; text arrives from the Frontend on a debounce and the engine persists the draft in SQLite
- [ ] The account's signature inserted into new drafts
- [ ] Send queues through the offline queue; send state as a `oneof` (queued, sending, sent, failed)
- [ ] Attachments: on desktop the engine reads the picked file by path so the bytes never enter the JS heap, then uploads per TODO-backend: attachment upload
- [ ] AI compose view: model choice, quick replies, tone, instruction, and generation as a `oneof` (empty, writing, draft ready, failed) with the draft as paragraphs; calls TODO-backend: `ComposeService.Write`
- [ ] Approve and send, edit and decline on an agent draft go through the queue to TODO-backend: `ApprovalService`
      The engine relays the decision and makes none of its own

## Search

- [ ] Decide: search index. Options: SQLite FTS5 (in bundled rusqlite, works in WASM), tantivy (better ranking, larger binary and memory). Blocks indexing
- [ ] Index subject, sender, recipients and body text during sync, in the same transaction as the row
- [ ] Search query, filters and results as engine state, served through the same windowed list
- [ ] Search tokenizer and ranking tests on fixed fixtures
- [ ] Results for mail outside the replica from TODO-backend: server search RPC, if it exists

## Agents and approvals

- [ ] `ui.v1.AgentsView`: agents with status as a `oneof` (running, done, waiting, paused, failed) and next run; the selected agent's schedule, model, instructions, stats and recent runs with steps
- [ ] Approval queue view: items with agent, time, title, detail and reason, each item's state as a `oneof` (waiting, resolving, done, declined), and the waiting count for the sidebar
- [ ] Intents for approve, edit, decline, run now, pause and resume, sent to TODO-backend: `AgentService` and `ApprovalService`
- [ ] `ui.v1.AgentBuilderView`: name, instructions, trigger, schedule options, accounts, Off/Ask/Auto per action, the new-recipient toggle, and the dry run as a `oneof` (idle, reading N messages across M accounts, results, failed)
- [ ] The unfinished new agent persisted as an engine draft until created or cancelled
- [ ] Agent and approval changes reach an open app through the sync protocol
      No permission or approval decision is duplicated from the Backend

## Settings and rules

- [ ] `ui.v1.SettingsView` with sections Accounts, Agent permissions, Rules, Models, Appearance; the selected section is engine state
- [ ] Accounts section: status ("Synced 1 min ago", "Sign-in expired"), signature, reconnect for an expired sign-in, add account
- [ ] Rules section with enabled toggles and new rule, sent to TODO-backend: rules
- [ ] Notify, evening digest, pause all, default permissions and models per task, sent to TODO-backend: `SettingsService`
- [ ] Theme (System, Dark, Light) and list density (Compact, Comfortable, Roomy) as engine-owned preferences persisted locally and sent in the shell view

## Web parity

- [ ] Decide: web local storage and which context owns the database with several tabs open. Options: SQLite in WASM on OPFS owned by a SharedWorker; one dedicated Worker elected with Web Locks and other tabs proxying to it; no persistent replica on web. Blocks the web replica
- [ ] Decide: how bodies reach a sandboxed iframe on web, where the custom URI scheme does not exist. Options: a Service Worker answering a body URL by asking the engine Worker; blob URLs created in the Worker; bodies served by the Backend. Blocks reading on web
- [ ] Web storage behind the platform adapter per the decision; nothing above the adapter knows the target
- [ ] Streaming on web per the sync protocol decision, behind the platform adapter
- [ ] Session on web per the session credential decision

## Packaging and measurement

- [ ] SQLite `cache_size` and mmap limits set explicitly in one place
- [ ] Measure engine memory with a 50k-message replica and record it against the desktop budget (whole app measured in TODO-frontend)
- [ ] Record WASM bundle size in CI
- [ ] Mobile targets per TODO-frontend: Decide mobile
