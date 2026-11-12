# Backend roadmap

The Backend runs on the GCE VM: Bun, TypeScript and PostgreSQL with pgvector. It owns provider connectors, OAuth, jobs, agents, scheduling, rules, notifications, LLM calls and every permission decision, and it defines the `api.v1` contract. Everything it does on its own is a job (see [docs/BACKEND.md](docs/BACKEND.md)).
Other parts: [TODO-rust-core.md](TODO-rust-core.md) (Rust engine), [TODO-frontend.md](TODO-frontend.md) (Frontend).
Tests sit next to the logic they cover and only cover hard, deterministic logic: pure functions and queries with fixed inputs, no timing.

## Walking skeleton

- [x] Bun.serve with handlers as pure functions in one RPC table behind a thin HTTP adapter
- [x] `POST /api.v1.Service/Method` with `application/proto` bodies, the Connect unary wire shape without a framework
- [x] `HealthService.Check` runs `SELECT 1` through Bun.sql, no ORM
- [x] Dev Postgres with pgvector in a tuned container (small `shared_buffers`, few `max_connections`, JIT and parallel workers off)
- [x] buf and protobuf-es codegen with generated code not committed, Biome, one CI workflow
- [x] Unknown path, wrong method, wrong content type and undecodable body each map to one explicit status, with tests of the HTTP adapter
- [x] CI runs `buf lint` for `api.v1` and `ui.v1`
- [ ] CI runs `buf breaking` against the main branch

## Foundations

- [ ] Postgres from the distribution's packages on the GCE VM with the tuned config
- [ ] Measure Postgres idle memory on the VM (anonymous plus shared memory of all its processes, not page cache)
      Gate: under 100 MB keeps Postgres; over it switches to Turso before the first table exists
- [ ] Plain hand-written `.sql` migrations for Postgres, applied in order at startup and recorded in one table
- [ ] Sign-in with Google or Microsoft through OIDC (authorize URL, callback, ID token check); the EveryAI account is added later as another provider of the same flow
- [ ] User and session tables, with `AuthService` in `api.v1`
- [ ] The session is a bearer credential for native clients and an HttpOnly, Secure, SameSite cookie for web, same origin as the Backend; both resolve to the same session row
- [ ] The RPC table marks each method as authenticated or public; handlers receive an authenticated user or never run

## Jobs, time and side effects

- [ ] Jobs table: user id, kind, status, due time, lease expiry and the other columns the claim query needs; the rest of the payload is a protobuf blob
      Every row carries a user id
- [ ] Job status as an explicit enum (scheduled, ready, running, waiting for approval, waiting for an event, paused, succeeded, failed, cancelled) with one pure transition function; tests for every state and event pair
      Waiting for an event covers a tool result, a provider batch result and another agent finishing
- [ ] Batch claim with `FOR UPDATE SKIP LOCKED` moves ready jobs to running under a lease and commits before any external call; test of ordering by priority then due time
      No transaction and no executor is held open across an external call: claim and commit, call, then record and commit
- [ ] A lease that expires returns the job to ready through the transition function, so a restart retries in-flight jobs from their start; tests with fixed times passed in
- [ ] Wake-up without idle polling: enqueueing signals the executor in process, and a timer sleeps until the next due time
      The timer delay is computed by Postgres (due time minus `now()`), not by the process clock. No `LISTEN/NOTIFY`: `Bun.sql` cannot receive it, so it is added only if the executor becomes a separate process
- [ ] Lanes in one registry in code: per job kind a concurrency cap, priority, retry policy and timeout; test that a full agent lane does not stop a send from being claimed
- [ ] Retry policy as a pure backoff function; a timeout counts as a failed attempt; an exhausted job becomes failed; tests
- [ ] Run log table for agent runs; a finished job is deleted in the same transaction that writes its outcome to the run log
- [ ] A job of another kind that exhausts its retries is shown on what it concerns: a fetch or token job sets the account status, a send job sets the message's send state, each with a reason
- [ ] Retention cleanup job: run log rows older than 90 days and transcripts older than 30 days are deleted, except the transcript of a parked run until it resolves; test of the cutoff function
- [ ] Fair claiming across users (round robin over users with ready jobs) and a per-user cap on running jobs; tests of fairness and ordering with one user holding a large backlog
- [ ] Executor interface inside the Backend process: take a claimed job, return an outcome, hold nothing between jobs; one in-process implementation, so a separate process can replace it later
- [ ] Postgres `now()` is the only source of "now": CI lint bans `Date.now()` and `new Date()` without arguments in the Backend
- [ ] Scheduled times arrive in `api.v1` as a `oneof` of an absolute UTC instant or wall time plus IANA zone; the server converts and stores a UTC `timestamptz`; tests of conversion across DST gap and overlap
- [ ] Durations are integer seconds, named with a `_seconds` suffix in protobuf and columns
- [ ] Recurrence rules stored with their zone; one pure next-run function for every schedule kind (every 15 minutes, every hour, daily at a time, weekdays at a time, custom); tests across DST changes
- [ ] Proactive interval minimum of 900 seconds, rejected on agent create and update, with seeded jitter in the next-run function; tests of the bound and the jitter range
- [ ] Missed-run policy: after downtime a schedule runs once, not once per missed slot; test in the next-run function
- [ ] Overlap policy: if the previous run is still going when the next is due, the next is skipped; test
- [ ] A job with a due time can be edited or cancelled until claimed, as one conditional update on status scheduled; test that an edit after the claim fails
- [ ] Side-effect ledger: idempotency key, status enum (pending, completed, failed) and recorded result
- [ ] One function runs every side effect through the ledger: pending row committed, call, completed row committed; a replayed completed key returns the recorded result without calling; ledger replay tests
- [ ] Events table: user id, a per-user increasing cursor, kind, subject, cause and hop count, written only through a helper that takes the transaction of the change that caused it
      The same table drives triggers and the sync stream, so a change is recorded once
- [ ] Logger module that writes to the system journal and accepts only an event kind, ids and durations: never mail content, prompts or tokens
      A type-level test that a free-text payload does not compile, and a CI lint that bans `console` outside the module

## First account connected and mail synced read-only

- [ ] Streamed response support in the HTTP adapter: the handler returns an async sequence of frames and the adapter writes each as a length-prefixed Connect envelope
- [ ] Mail connector interface in TypeScript: list mailboxes, fetch changes since a cursor, fetch a message, apply a change (move, label, star, delete, report as spam)
- [ ] Three implementations behind it: Gmail API, Microsoft Graph, and IMAP with SMTP for everything else
- [ ] Provider tokens, passwords and LLM keys encrypted at rest by one encrypt and decrypt module with a key kept outside the database
- [ ] Google OAuth for Gmail: authorize URL, callback, token storage and refresh, sharing the OIDC code of sign-in
- [ ] Google OAuth app in testing mode with the named test users added in Google's console (limit 100); Google's refusal of any other user maps to the account's failed status with a reason
- [ ] Start Google verification for restricted Gmail scopes and its security assessment, running in parallel with testing mode
- [ ] Microsoft OAuth for Outlook (Microsoft 365 and outlook.com), sharing the OIDC code of sign-in; open to everyone from the start
- [ ] Start Microsoft publisher verification
- [ ] First sign-in can connect that provider's mailbox in the same step; connecting a mail account stays a separate call
- [ ] Token refresh as a job kind, due before expiry
- [ ] IMAP with password for iCloud Mail, Yahoo Mail and Fastmail (app-specific password) and Other IMAP (server and password); open to everyone from the start
- [ ] `AccountService` in `api.v1`: start connect (returns the provider sign-in URL), finish connect, list, disconnect (purges per User data)
- [ ] Account status as a `oneof`: syncing with progress (messages, folders), synced at a time, sign-in expired, failed with a reason
- [ ] One mailbox model in `api.v1` covering Gmail labels, Graph folders and IMAP folders (Starred, Sent, Archive and user folders such as Receipts)
- [ ] Mail tables: metadata and thread structure for all mail, extracted text for the last 90 days
- [ ] Generated drift check in CI: fails when a synced message in `api.v1` and its Postgres or SQLite tables drift apart (needs TODO-rust-core: hand-written SQLite migrations)
- [ ] Eviction job deletes extracted text older than 90 days and never touches vectors; older bodies and all attachments are fetched from the provider on demand
- [ ] Fetch is a job kind in its own lane: initial sync fetches newest mail first and reports progress, so the inbox opens before the backfill ends
- [ ] Incremental fetch per account from a stored provider cursor (Gmail history id, Graph delta link, IMAP UIDVALIDITY with MODSEQ or UID)
- [ ] Tests of each connector's cursor-to-changes logic on recorded fixtures, including a UIDVALIDITY reset
- [ ] Gmail watch over Pub/Sub, created at connect and renewed by a job before it expires
- [ ] Microsoft Graph subscriptions, created at connect and renewed by a job before they expire
- [ ] Webhook endpoints for Gmail push and Graph notifications: accept JSON, verify the sender, then only enqueue a fetch job for that account, deduplicated while one is waiting
      The providers send JSON, the allowed exception to protobuf only
- [ ] Polling fetch jobs every 60 seconds for accounts without push (IMAP and the rest), each re-enqueued with a due time; no held IMAP IDLE connections
- [ ] Fetch writes the new mail arrived event in the same transaction as the message row
- [ ] The mail moved to a folder event and the label or star added event are written in the same transaction as the change, whether it came from a fetch or from a `MailService` mutation
- [ ] The Backend parses MIME once at ingest into parts the engine receives, tested on a corpus of real-world messages (nested multipart, broken charsets, inline images)
- [ ] `SyncService.Stream` in `api.v1`: opened with a cursor, answers with a streamed response of frames (snapshot batch, change batch, heartbeat or resync as a `oneof`); catch-up and live updates are the same call
- [ ] Snapshot for a client with no cursor: the stream first sends the newest mail, newest first, then the live changes; `SyncService.ListOlder` is a unary call that pages older mail before a position
- [ ] Per-user fan-out: an open stream wakes on the in-process signal after an event commits and sends the events after its cursor; no polling
- [ ] Heartbeat frame every 25 seconds on an idle stream
- [ ] Cursor semantics: resuming from a cursor returns exactly the later events once, in order; a cursor older than the oldest kept event gets a resync frame; tests including a repeated cursor and a cursor from before the oldest kept event
- [ ] Events older than 30 days are deleted by the retention cleanup job
- [ ] Tests of frame encoding: envelope flags, length prefix and a message split across reads

## LLM access and limits

- [ ] `ModelService` in `api.v1`: set and remove a provider key, list providers with key status and the models they offer; a key is never returned
- [ ] Provider adapter interface: one call with messages, tools and a cache boundary, returning text, tool calls and token usage; plus batch submit and collect
- [ ] Anthropic adapter
- [ ] OpenAI adapter
- [ ] Google Gemini adapter
- [ ] OpenRouter adapter
- [ ] Adapter tests on recorded fixtures: request mapping, tool call parsing and usage counting
      Adding a dedicated provider later is one more implementation of the interface
- [ ] Batch API use for work that can wait: a submit job leaves the job waiting for an event, and a collect job with a due time finishes it; used by the briefing
- [ ] Batch submit refuses calls made for Gatekeeper and Security Guard, which always use the single-call path; test
- [ ] Token usage recorded per user for every call (kind, model, input, output and cached tokens), dry runs included, so a plan can attach to it later; no billing code
- [ ] Fixed safety caps for everyone as constants in one module (30 steps per run, 200 runs per agent per day, 3 concurrent runs per user), and an optional daily token budget per user, unset by default; one pure cap-check function; tests at, below and over each cap
- [ ] Token budget checked before each LLM call; at the budget LLM calls stop while the mail fetch, send and manual action lanes keep working, with a test
- [ ] Key state per user as an explicit enum (no key, key set); with no key, LLM steps (Scribe, AI compose, user agents) are off and calls are refused with that state, which `ModelService` returns for the engine; system agents still run their code stage

## Inbox and thread reading

- [ ] `MailService.GetBody` returns a message's parsed parts, fetching from the provider when the text is no longer cached; the engine calls it when it serves an older body
- [ ] `MailService.GetAttachment` fetches an attachment's bytes from the provider by attachment id; the engine serves them
- [ ] Threading per account: provider thread ids where they exist, `References` and `In-Reply-To` for IMAP, with tests
- [ ] EveryMail categories (For you, Updates, Reading as tabs), the Quarantine folder and user-created folders as tables, with a category or folder assignment per thread synced to the engine; nothing is moved at the provider
- [ ] `MailService` calls to create, rename and delete a user folder and to move a thread into a category or folder
- [ ] Row note ("Inbox sweeper drafted a reply", "Filed to Receipts") as a synced field, set only by the agent or rule that acted

## Embeddings

- [ ] Platform embedding key read from the server environment, outside the database; embeddings are the one use of a platform key
- [ ] One embedding call in a small module: the Gemini embedding API, vectors shortened to 768 dimensions and normalized, the only call the platform pays for; it is not part of the user-key adapter interface
      Test of the normalization and the request mapping on a recorded fixture
- [ ] Vector table with `vector(768)` and a pgvector index, one row per message, kept after its text is evicted
- [ ] Embedding as a job kind in its own lane, enqueued by the new mail arrived event; every user gets it whatever key they brought
- [ ] A hit on older mail fetches the body from the provider through `MailService.GetBody`

## Write actions

- [ ] `MailService` mutations in `api.v1`: archive, star, unstar, mark read, mark unread, move, delete; each carries a client-generated idempotency key used as its ledger key
- [ ] Mutations apply to the provider through the side-effect ledger and return the resulting state; a repeated idempotency key returns the first result, with a test
- [ ] `MailService.Snooze` takes a preset (Later today, Tomorrow, This weekend, Next week) with the user's IANA zone, or a picked wall time plus zone, or an instant; the server resolves it with its own clock and stores a due-time job
- [ ] The snooze job returns the thread to the inbox at its due time with every device off; edit and cancel through the due-time rule
- [ ] Preset resolution as a pure function of server now and zone: Later today is 18:00 or 3 hours from now once past 15:00, Tomorrow 08:00, This weekend Saturday 09:00, Next week Monday 08:00; tests across DST
- [ ] Decide: what to offer for Google accounts while Gmail is in testing mode. Options: agents on Gmail accounts only draft and never send until verification; drafts written into Gmail's Drafts folder, which the Drafts rule otherwise forbids; nothing special. Blocks the Gmail send tasks and the draft replies action
- [ ] `MailService.Send` creates a send job: due 10 seconds after the Backend receives it for undo send, or at a given time for scheduled send; undo, reschedule and cancel go through the due-time rule
- [ ] The send job runs in the send lane from the address the mail came to, with our own `Message-ID`, through the ledger
- [ ] After an uncertain send the job looks for its `Message-ID` in Sent; found means sent, not found means the job waits for approval with an item asking the user; a send is never retried blindly; tests of the decision function
- [ ] Reply and forward set `In-Reply-To`, `References` and the provider thread id, with tests per provider
- [ ] Attachment upload in `api.v1` with a size limit, attached on send
- [ ] Per-account signature stored in settings and synced
- [ ] `ComposeService.Write` in `api.v1`: thread, quick reply or free instruction, tone (Brief, Warm, Formal), model; returns a draft in the user's voice
- [ ] Drafts table and `DraftService` in `api.v1`: save, list, delete; every draft change is an event on the sync stream, so drafts reach the user's other devices and are visible to agents; nothing is written to the provider's Drafts folder unless the Decide above says so
- [ ] Recall of the user's own writing by embedding search feeds `ComposeService.Write` so drafts sound like the user

## Search

- [ ] Search by meaning for the user over embeddings, as one RPC; it also reaches mail outside the engine's local replica

## Agent runs

- [ ] Run tables: run id, agent, cause and hop count, status as a `oneof` (running, waiting for approval, paused, done, declined, failed with a reason, stopped at a limit), and a transcript of steps (kept, drafted, archived, labelled, filed, logged, failed); retention per the cleanup job above
- [ ] Tool registry in code: each tool has a typed input, an action category for permissions and an executor; the model's tool list is generated from it
- [ ] Tools offered to the model are only those whose category is not Off; a test that an Off category is absent from the list
- [ ] The one fixed system prompt, versioned in code; the version is stored on each run
- [ ] Prompt assembly as a pure function from stored parts, in order: system prompt, tools, the agent's instructions, permissions, notes, mail content and other context; the adapter marks the end of the stable prefix; tests of the order and that the prefix is identical across steps of one run
- [ ] Mail content and notes are wrapped as delimited data in the prompt; the wrapper neutralizes the delimiter inside content; test with hostile content
- [ ] "Advance run" job kind: load the transcript, make one LLM call, store the response, enqueue the tool calls; each tool result enqueues "advance run" again
- [ ] Tool calls run as jobs through the side-effect ledger; the run's job is waiting for an event until their results are stored
- [ ] Embedding retrieval tool: an agent finds related past mail by meaning
- [ ] A run stops at the steps per run cap or at 600 seconds of active time (time spent waiting for approval does not count); one pure limit function, with tests
- [ ] `finish` tool with a typed deliverable message; a reply with no tool call fails the run; prose is never parsed
- [ ] Three kinds of memory in three places: the transcript per run, notes, and the handover, which is only the typed deliverable and never the transcript
- [ ] Notes: one notebook per agent and one note per agent and thread, written through a tool, size-capped (test of the cap), and listed and edited by the user through `AgentService`
      Notes are untrusted: they enter the prompt only as delimited data
- [ ] Thread context: the latest messages in full plus the stored thread summary; tools fetch older messages and attachments on demand from the provider
- [ ] An agent cannot create or change agents or schedules, its own included: no tool does it, and `AgentService` accepts only user sessions
- [ ] One permission function returns Auto or Ask for a tool call at execution time from its setting, the run's read state, the recipient and the thread; tests for the new-recipient rule and for no Auto send to a non-participant of the thread after a mail body or note was read
- [ ] The run's read state is an explicit enum (clean, read mail or notes) stored on the run, set when any tool returns a mail body or a note
- [ ] An action that resolves to Ask parks the run: it creates an approval item and the job waits for approval; resolving the approval resumes it
- [ ] The approval expires after 7 days through the due-time rule and the run ends as declined; an edit made before approving is recorded in the transcript
- [ ] At a safety cap (runs per agent per day, concurrent runs) or the token budget new runs are not created; in-flight runs finish their step and park; the user is told why through the run status
- [ ] Dry run: the same executor with every write recorded instead of executed; its LLM calls count toward the caps
- [ ] Pause all: each run finishes its current step, then moves to paused; resume returns them to ready
- [ ] A run failing on an expired sign-in sets that account's status to sign-in expired and continues with the other accounts

## Agents and approvals

- [ ] `AgentService` in `api.v1`: create, update, pause, resume, delete, run now, list runs
- [ ] Agent profile: name, plain-words instructions, triggers (the reactive ones below, a lead time in seconds on calendar event approaching, and an optional schedule: every 15 minutes, every hour, daily at a time, weekdays at a time, custom), filters (sender, account, folder), model, accounts it can see, per-action permissions
- [ ] Per-action permission Off, Ask or Auto for read and label, update sheets, calendar, draft replies, send, archive, unsubscribe, delete, sort into a category or folder, move to Quarantine, block a sender, report as spam, notify me, mail the briefing; plus "always ask before writing to someone new", which overrides Auto for first-time recipients
- [ ] Matcher: turns events into "start run" jobs for subscribed agents, deduplicated by agent and event with a unique key; test of dedup
- [ ] Trigger filters evaluated in code before any LLM call
- [ ] Stop at depth 3: the matcher drops an event whose hop count reached 3, and an agent's own actions never wake itself; tests
- [ ] A burst of mail creates one run per mail, with one run at a time per agent (a per-agent concurrency condition in the claim); test
- [ ] Reactive trigger: new mail arrived (ordering in System agents)
- [ ] Reactive trigger: mail sorted into a category or moved to a folder
- [ ] Reactive trigger: mail flagged
- [ ] Reactive trigger: label or star added
- [ ] Reactive trigger: another agent finished, with the handover deliverable as the event payload
- [ ] Reactive trigger: approval resolved
- [ ] Proactive trigger: the schedule job per agent, which first runs a code-only check for anything new since the agent's last run and ends without any LLM call if nothing is new; test of the check
- [ ] An action set to Ask creates an approval item (agent, title, detail, reason such as "Send is set to Ask for new recipients") instead of acting
- [ ] `ApprovalService` in `api.v1`: list, approve (performs the action), edit then approve, decline
- [ ] Agent stats: handled this month, runs with no questions, waiting count, last run
- [ ] Dry run on the last 30 messages: one row per message with its action and Auto, Ask or Skip, returned by `AgentService`
- [ ] Unsubscribe action through `List-Unsubscribe` (one-click POST where offered, mailto otherwise)
- [ ] Pause all in `SettingsService` and `AgentService` calls the run pause above and stops every schedule until resumed

## Drive and OneDrive for agents

Agents only: files and sheets an agent is pointed at. There is no file browser

- [ ] Google Sheets and Drive scopes on the Google connection; OneDrive and Excel scopes on the Microsoft connection
- [ ] Sheet action for agents: add or update a row in a sheet named in the instructions (update sheets permission)
- [ ] Reactive trigger: file changed, from Drive and OneDrive change feeds by push where offered and polling jobs otherwise

## Rules and the block list

- [ ] Rules as a fixed set of conditions (sender domain, subject contains, is a newsletter, sender is new and mail has a link) and actions (mark important, notify, hand to an agent, skip the inbox into the briefing, flag as suspicious, star), each rule with an enabled flag; all evaluated in code, no plain-words conditions
- [ ] Rules run on every new mail arrived event in order, with tests of rule matching
- [ ] Block list table per user; mail from a blocked sender goes to Quarantine in code without any agent run; test
- [ ] `RulesService` in `api.v1`: list, create, edit, enable and disable, delete rules; list, add and remove block list entries

## System agents

- [ ] System flag on agents, and every new user gets the system agents (Gatekeeper, Security Guard, Scribe, News Bearer) in the same transaction as the user row; a migration provisions existing users
- [ ] Agent active state as an explicit enum (active, deactivated); `AgentService` refuses to delete a system agent and allows deactivating it; a user agent can be deleted; tests
- [ ] Built-in instructions fixed and versioned in code, never copied into rows; the user's appended instructions are stored apart and prompt assembly adds them after the built-in text; test of the assembly
- [ ] On a system agent `AgentService` accepts only per-action permissions, schedule and appended instructions; built-in defaults for each action live in code; tests of the rejected fields
- [ ] Built-in code stage interface: a pure step that runs before any LLM call and returns either a decision, which finishes the run without a call, or "needs the model"
- [ ] Without a key a system agent runs its code stage only; what the code cannot decide ends the run as undecided; test
- [ ] Order on a new mail: the matcher gives the new mail arrived event to other agents only when Gatekeeper's run for that mail has finished (any outcome), and at once when Gatekeeper is deactivated; tests for both and for rules running first
- [ ] Gatekeeper code stage as a pure function over bulk headers, the provider's spam verdict, SPF, DKIM and DMARC results, the block list and whether the user has written to the sender; tests on fixtures
- [ ] Gatekeeper code stage also uses vectors when they exist: the nearest already-sorted mail of the user suggests a category when the headers cannot decide; pure function over vectors with a test
- [ ] Gatekeeper model stage for what the code cannot decide, only with a key, one call per mail on the single-call path, never a batch API
- [ ] Gatekeeper tools: sort into a category or folder (writes the assignment and the sorted event) and flag as malicious or a scam (writes the flagged event)
- [ ] Deactivated Gatekeeper sorts and flags nothing: all mail lands in For you and Security Guard never wakes; the block list and the user's rules still apply; tests
- [ ] First-connect sweep job kind: Gatekeeper's code stage over the last 90 days of mail already in the account, sorting it with no model call and no run counted against the caps; test that no LLM call is made and the cap counters do not move
- [ ] Sweep asks once per bulk sender flooding the inbox whether to unsubscribe: one approval item per sender, unsubscribing on approval; test of the grouping function
- [ ] Security Guard wakes on the mail flagged event; its Auto actions are move to Quarantine, add the sender to the block list, report as spam to the provider, and notify the user of what it caught
- [ ] Security Guard never uses a batch API, and its tool list has no tool that follows a link or unsubscribes; tests that the list contains neither
- [ ] Reporting spam moves the mail to the provider's spam folder; the Quarantine cleanup job drops EveryMail's copy after 30 days, and the provider's retention applies to its spam folder
- [ ] Scribe runs lazily: `MailService` summary requests start a Scribe run the first time a thread summary is needed and return a pending state until it is stored; no trigger subscribes it to new mail, with a test
- [ ] Scribe writes the message summaries, the thread summary and the quick reply suggestions of a thread in one run, stored as synced fields; the thread context tool of any agent starts Scribe when the summary is missing and the run waits for an event
- [ ] News Bearer on a user-set schedule: a code stage collects what agents did (run log), what waits, the day's calendar and the newsletters and updates, and the model condenses them, through the batch API
- [ ] Briefing record and `BriefingService` in `api.v1` to list and read briefings, synced to the engine
- [ ] A permission on News Bearer lets it also mail the briefing, through the send path
- [ ] Later, not designed now: News Bearer adds outside news through EveryAI's web search tool

## Notifications

- [ ] Notification job kind, one for every channel; notified events are an approval waiting, Security Guard caught something, a briefing is ready, and mail an agent was told to report at once
- [ ] The notify me action raises a notification for mail an agent was told to report at once, under its permission
- [ ] Desktop: the notification is an event on the sync stream, which the engine raises as a system notification while the app runs
- [ ] Web Push: subscription stored per device through `NotificationService`, and the Backend sends the push
- [ ] APNs and FCM: device tokens registered per device through `NotificationService`, and the Backend sends to both

## Calendar

- [ ] Calendar connector interface: list calendars, fetch changes since a cursor, create, edit and delete an event, answer an invitation
- [ ] Three implementations behind it: Google Calendar, Outlook calendar, and CalDAV for iCloud and Fastmail
- [ ] Google Calendar and Outlook calendar scopes on the Google and Microsoft connections; CalDAV credentials encrypted with the other provider secrets
- [ ] Calendar tables for calendars and events, with instants in UTC and recurrence rules with their zone
- [ ] Tests of each connector's change mapping on recorded fixtures and of recurring events across DST
- [ ] Calendar fetch as a job kind with push where offered and polling jobs otherwise
- [ ] `CalendarService` in `api.v1`: list a range, create, edit, delete and respond to an invitation; every write goes through the side-effect ledger
- [ ] Calendar changes are events on the sync stream
- [ ] Invitations detected in mail at ingest from the iCalendar part, as a field on the message that links to the event and to `CalendarService` respond; tests on fixtures
- [ ] Calendar action for agents: read free time and hold a slot (calendar permission)
- [ ] Reactive trigger: calendar event approaching, a due-time job created when the event is synced at event start minus the trigger's lead time, writing its event when it comes due

## Settings

- [ ] `SettingsService` in `api.v1`: default permissions for new agents (each agent overrides its own), pause all, briefing times, model per task (writing replies, summaries, agents), time zone, usage against caps and the daily token budget (signatures per account are under Write actions)

## User data

- [ ] One purge function, called by account deletion and by mailbox disconnect: cancels the jobs, revokes the provider tokens through the ledger, deletes the data from Postgres, and changes nothing at the provider
      Mailbox disconnect purges that mailbox's mail, vectors, assignments and jobs; account deletion purges the user and ends every session
- [ ] Test that after a purge no row of that user or mailbox remains in any table; a CI check that every table carries a user id or sits on an explicit list of global tables
- [ ] `AuthService` call to delete the account, which runs the purge
- [ ] `UserDataService.Export` in `api.v1` returns one file of what only EveryMail holds: agents and their instructions, notes, rules, the block list and the run log; mail is not exported; test of the assembly on fixtures

## Waitlist

- [ ] Waitlist table holding the fields of the landing form, unique per email
- [ ] Public RPC that adds a signup; it is the single call a client other than the engine makes
- [ ] Rate limiting on the public RPC as a pure function over request counts per client address and a fixed window passed in; tests at, below and over the limit

## Deployment and measurement

- [ ] Bun as a systemd service on the GCE VM next to the EveryAI services, no containers in production
- [ ] The reverse proxy that serves EveryAI routes mail.everyai.dev with TLS to the Backend, the landing site and the web app (per TODO-frontend: Decide where the web app is served), reaches the webhook endpoints from Gmail push and Graph, and does not buffer the sync stream; a heartbeat arrives through it within 25 seconds
- [ ] Encryption key is a root-only file on the VM outside Postgres; the Backend refuses to start without it
- [ ] Watchdog job kind on a due-time schedule: a pure threshold function over queue depth, the oldest ready job and failure rates, and a notification to the owner when a threshold is crossed; test of the function; no external monitoring service
- [ ] Revisit backups before the first real users: none exist until then, and agents, notes, rules and run logs are lost with the VM
- [ ] Measure Backend idle and loaded memory on the VM and record it next to the Postgres gate
- [ ] Measure mail arrival to the start of an agent run against the targets: under 60 seconds with push, under 120 with polling

## Anole swap

- [ ] Anole supports a streamed response body, checked with the sync stream before anything else
- [ ] Replace the Bun.serve adapter with Anole; the RPC table and handlers stay unchanged
- [ ] Run the owner's benchmark routes against both adapters on the VM and record req/s, p99, CPU per request and memory
- [ ] Confirm Backend plus Postgres memory still fits the VM with Anole's higher idle cost (about 100 MB more in the owner's benchmark)
