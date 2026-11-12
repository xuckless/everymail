# Backend roadmap

The Backend runs on the GCE VM: Bun, TypeScript and PostgreSQL with pgvector. It owns provider connectors, OAuth, agents, scheduling, rules, notifications, LLM calls and every permission decision, and it defines the `api.v1` contract.
Other parts: [TODO-rust-core.md](TODO-rust-core.md) (Rust engine), [TODO-frontend.md](TODO-frontend.md) (Frontend).

## Walking skeleton

- [x] Bun.serve with handlers as pure functions in one RPC table behind a thin HTTP adapter
- [x] `POST /api.v1.Service/Method` with `application/proto` bodies, the Connect unary wire shape without a framework
- [x] `HealthService.Check` runs `SELECT 1` through Bun.sql, no ORM
- [x] Dev Postgres with pgvector in a tuned container (small `shared_buffers`, few `max_connections`, JIT and parallel workers off)
- [x] buf and protobuf-es codegen with generated code not committed, Biome, one CI workflow
- [x] Unknown path, wrong method, wrong content type and undecodable body each map to one explicit status, with tests of the HTTP adapter
- [x] CI runs `buf lint` for `api.v1` and `ui.v1`
- [ ] CI runs `buf breaking` against the main branch

## Decisions that block everything

- [ ] Measure Postgres idle memory on the GCE VM with the tuned config (anonymous plus shared memory of all its processes, not page cache)
      Gate: under 100 MB keeps Postgres; over it switches to Turso before the first table exists
- [ ] Decide: schema ownership. Options: protobuf as the source generating Postgres and SQLite DDL; per-store SQL migrations with protobuf only on the wire plus a generated shape check; one small schema file generating all three. Blocks the first table here and in the Rust engine
- [ ] Decide: auth. Options: EveryAI identity (the design says "Part of EveryAI"), own passkey or email-link sessions in Postgres, a hosted provider. Blocks every user-scoped RPC and the engine session
- [ ] Decide: what the Backend stores per message. Options: full parsed mail in Postgres; metadata with bodies fetched from the provider on demand; metadata plus a bounded body cache. Blocks the mail tables and the sync protocol
- [ ] Decide: where MIME parsing and HTML sanitizing happen. Options: the Backend parses once at ingest and syncs parsed parts; the engine parses and agents get text another way. AGENTS.md assigns parsing to the engine, but agents in TypeScript need parsed mail, so exactly one side owns it. Blocks ingest and body serving
- [ ] Decide: job scheduling. Options: a Postgres jobs table claimed with `FOR UPDATE SKIP LOCKED` inside the Backend process, a library on top of Postgres, an external scheduler. Blocks provider polling, token refresh, snooze wake-ups, agent schedules and the digest
- [ ] Plain `.sql` migrations applied in order at startup and recorded in one table
- [ ] User and session tables per the auth decision, with `AuthService` in `api.v1`
- [ ] The RPC table marks each method as authenticated or public; handlers receive an authenticated user or never run
- [ ] Jobs table and worker loop per the scheduling decision, with tests of claim, retry and backoff ordering

## First account connected and mail synced read-only

- [ ] Decide: protocol per provider. Options: Gmail API and Microsoft Graph for Gmail and Outlook with IMAP for the rest; IMAP and SMTP with XOAUTH2 for all. Blocks the connector interface
- [ ] Decide: how the Backend learns about new mail. Options: provider push (Gmail watch over Pub/Sub, Graph subscriptions), held IMAP IDLE connections, polling through the scheduler. Blocks "on new mail" triggers and sync latency
- [ ] Decide: Proton Mail. Proton Bridge runs on the user's machine, so a Backend on GCE cannot reach it. Options: drop it; a desktop-only path (conflicts with the engine being the Backend's only client and with agents running while devices are off). Blocks the Proton connector
- [ ] Connector interface in TypeScript: list mailboxes, fetch changes since a cursor, fetch a message, apply a change; one implementation per protocol
- [ ] Google OAuth for Gmail: authorize URL, callback, token storage and refresh
- [ ] Microsoft OAuth for Outlook (Microsoft 365 and outlook.com)
- [ ] Start Google verification for restricted Gmail scopes (security assessment) and Microsoft publisher verification
- [ ] IMAP with password for iCloud Mail (app-specific password) and Other IMAP (server and password)
- [ ] Yahoo Mail and Fastmail sign-in
- [ ] Provider tokens and passwords encrypted at rest with a key kept outside the database
- [ ] `AccountService` in `api.v1`: start connect (returns the provider sign-in URL), finish connect, list, remove
- [ ] Account status as a `oneof`: syncing with progress (messages, folders), synced at a time, sign-in expired, failed with a reason
- [ ] One mailbox model in `api.v1` covering Gmail labels, Graph folders and IMAP folders (Starred, Sent, Archive and user folders such as Receipts)
- [ ] Initial sync fetches newest mail first and reports progress, so the inbox opens before the backfill ends
- [ ] Incremental sync per account from a stored provider cursor (Gmail history id, Graph delta link, IMAP UIDVALIDITY with MODSEQ or UID)
- [ ] Tests of each connector's cursor-to-changes logic on recorded fixtures, including a UIDVALIDITY reset
- [ ] MIME parsing on the side chosen above, tested on a corpus of real-world messages (nested multipart, broken charsets, inline images)
- [ ] Backend half of the sync protocol (needs TODO-rust-core: Decide sync protocol)

## Inbox and thread reading

- [ ] Threading per account: provider thread ids where they exist, `References` and `In-Reply-To` for IMAP, with tests
- [ ] Decide: how mail is sorted into the For you, Updates and Reading tabs. Options: provider categories, rules, an LLM classifier, a mix. Blocks tab assignment
- [ ] Tab assignment stored per thread and synced
- [ ] Decide: LLM access. Options: EveryAI's model gateway with the keys set up there (the design's Settings says so), direct provider SDKs, user-supplied keys. The design offers Claude Sonnet, GPT-5, Gemini Pro, Grok and a "Local model"; a local model cannot serve agents that run while devices are off. Blocks summaries, AI compose and agents
- [ ] Message summary and thread summary generated once per new message with the Summaries model, synced as fields
- [ ] Row note ("Inbox sweeper drafted a reply", "Filed to Receipts") as a synced field, set only by the agent or rule that acted

## Write actions

- [ ] `MailService` mutations in `api.v1`: archive, star, unstar, mark read, mark unread, move, delete, snooze; each carries a client-generated idempotency key
- [ ] Mutations apply to the provider and return the resulting state; a repeated idempotency key returns the first result, with a test
- [ ] Snooze stored with an absolute wake time; the scheduler returns the thread to the inbox at that time with every device off
- [ ] Send through the provider per the protocol decision, from the address the mail came to
- [ ] Reply and forward set `In-Reply-To`, `References` and the provider thread id, with tests per provider
- [ ] Attachment upload in `api.v1` with a size limit, attached on send
- [ ] Per-account signature stored in settings and synced
- [ ] `ComposeService.Write` in `api.v1`: thread, quick reply or free instruction, tone (Brief, Warm, Formal), model; returns a draft in the user's voice
- [ ] Quick reply suggestions per thread generated alongside the thread summary

## Search

- [ ] Decide: what pgvector is for. Options: semantic search over mail outside the local replica, agent retrieval over past mail, neither (drop the extension). Blocks any embedding pipeline
- [ ] Server search RPC for mail outside the local replica, only if TODO-rust-core: Decide replica depth leaves mail out of it

## Agents and approvals

- [ ] `AgentService` in `api.v1`: create, update, pause, resume, delete, run now, list runs
- [ ] Agent profile: name, plain-words instructions, trigger (new mail, schedule, both), schedule (every 15 minutes, every hour, daily at a time, weekdays at a time, custom), model, accounts it can see, per-action permissions
- [ ] Per-action permission Off, Ask or Auto for read and label, update sheets, calendar, draft replies, send, archive, unsubscribe, delete; plus "always ask before writing to someone new", which overrides Auto for first-time recipients
- [ ] One permission function decides Auto, Ask or Off for every agent action, with tests including the new-recipient override
- [ ] Next-run computation for each schedule kind in the user's time zone, with tests across DST changes
- [ ] Agent runner: claims a due job, offers the model only actions not set to Off, records each step
- [ ] Run record: start, duration, outcome as a `oneof` (done, waiting for approval, failed with a reason such as an expired sign-in), and steps (kept, drafted, archived, labelled, filed, logged, failed)
- [ ] A run failing on an expired sign-in sets that account's status to sign-in expired and continues with the other accounts
- [ ] An action set to Ask creates an approval item (agent, title, detail, reason such as "Send is set to Ask for new recipients") instead of acting
- [ ] `ApprovalService` in `api.v1`: list, approve (performs the action), edit then approve, decline
- [ ] Agent stats: handled this month, runs with no questions, waiting count, last run
- [ ] Dry run on the last 30 messages: the same runner with every write replaced by a recorder, returning one row per message with its action and Auto, Ask or Skip
- [ ] Unsubscribe action through `List-Unsubscribe` (one-click POST where offered, mailto otherwise)
- [ ] Pause all agents stops every schedule until resumed
- [ ] Default permissions for new agents in settings; each agent overrides its own
- [ ] Decide: notification delivery for "Notify me when something waits". Options: Web Push for web; APNs and FCM once mobile is decided; desktop notifications through the engine only while the app runs. Blocks notifications
- [ ] Notify on a new approval item and on mail an agent is told to report right away
- [ ] Decide: what the evening digest is. Options: an email to the user, an in-app view, both; content is what agents did today plus mail routed to "tonight's digest". Blocks the digest job
- [ ] Evening digest job at the user's configured time (21:00 in the design)

## Drive, OneDrive and calendar

- [ ] Decide: what Drive, OneDrive and calendar expose beyond agent actions. AGENTS.md promises read/write access; the design shows only agent use (sheet rows, calendar holds, "added to calendar"). Blocks any Drive or calendar screen in the other parts
- [ ] Google Sheets, Drive and Calendar scopes on the Google connection; OneDrive, Excel and Outlook calendar scopes on the Microsoft connection
- [ ] Sheet action for agents: add or update a row in a sheet named in the instructions (update sheets permission)
- [ ] Calendar action for agents: read free time and hold a slot, for Google Calendar and Outlook calendar (calendar permission)

## Settings and rules

- [ ] `SettingsService` in `api.v1`: signatures, default agent permissions, notify, digest, pause all, model per task (writing replies, summaries, agents), time zone
- [ ] Decide: rule authoring. Options: a fixed set of conditions and actions; plain-words conditions read by an LLM (the design shows "mail mentions an interview"). Blocks the rules tables
- [ ] Rules: conditions (sender domain, subject contains, is a newsletter, sender is new and mail has a link) and actions (mark important, notify, hand to an agent, skip the inbox into the digest, flag as suspicious, star), each rule with an enabled flag
- [ ] Rules run on every new message in order, before any agent, with tests of the matcher

## Deployment and measurement

- [ ] Decide: how the Backend runs on the VM. Options: a systemd unit running Bun, one container next to Postgres. Blocks the first deploy
- [ ] Deploy to the GCE VM next to the EveryAI services, with TLS on the domain (mail.everyai.dev in the design)
- [ ] Postgres backups with a tested restore
- [ ] Measure Backend idle and loaded memory on the VM and record it next to the Postgres gate
- [ ] Waitlist signup RPC (needs TODO-frontend: Decide landing page and waitlist)

## Anole swap

- [ ] Replace the Bun.serve adapter with Anole; the RPC table and handlers stay unchanged
- [ ] Run the owner's benchmark routes against both adapters on the VM and record req/s, p99, CPU per request and memory
- [ ] Confirm Backend plus Postgres memory still fits the VM with Anole's higher idle cost (about 100 MB more in the owner's benchmark)
